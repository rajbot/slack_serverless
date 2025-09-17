#[cfg(feature = "lambda")]
use crate::app::App;
use crate::error::{Result, SlackError};
use crate::request::{SlackRequest, SlackRequestBody, EventRequest, CommandRequest, InteractiveRequest, OAuthRequest};
use crate::response::SlackResponse;
use crate::context::Context;
use crate::client::SlackClient;
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use urlencoding::decode;
use tracing::{info, error, warn};

#[derive(Clone)]
pub struct LambdaHandler {
    app: App,
}

impl LambdaHandler {
    pub fn new(app: App) -> Self {
        Self { app }
    }

    pub async fn run(self) -> std::result::Result<(), LambdaError> {
        lambda_runtime::run(service_fn(move |event| {
            let handler = self.clone();
            async move { handler.handle_request(event).await }
        }))
        .await
    }

    async fn handle_request(&self, event: LambdaEvent<ApiGatewayProxyRequest>) -> std::result::Result<ApiGatewayProxyResponse, LambdaError> {
        let (request, _context) = event.into_parts();
        
        match self.process_request(request).await {
            Ok(response) => Ok(self.to_api_gateway_response(response)),
            Err(e) => {
                error!("Error processing request: {}", e);
                Ok(ApiGatewayProxyResponse {
                    status_code: 500,
                    headers: HashMap::new(),
                    body: Some("Internal Server Error".to_string()),
                    is_base64_encoded: false,
                })
            }
        }
    }

    async fn process_request(&self, request: ApiGatewayProxyRequest) -> Result<SlackResponse> {
        let slack_request = self.to_slack_request(request)?;
        
        // Verify request signature
        if let Err(e) = self.verify_signature(&slack_request) {
            warn!("Invalid request signature: {}", e);
            return Ok(SlackResponse {
                status_code: 401,
                headers: HashMap::new(),
                body: crate::response::SlackResponseBody::Empty,
            });
        }

        // Handle different request types
        match &slack_request.body {
            SlackRequestBody::Event(event_req) => {
                // Handle URL verification challenge
                if let Some(challenge) = &event_req.challenge {
                    return Ok(SlackResponse::challenge(challenge));
                }
                
                self.handle_event_request(slack_request).await
            }
            SlackRequestBody::Command(_) => {
                self.handle_command_request(slack_request).await
            }
            SlackRequestBody::Interactive(_) => {
                self.handle_interactive_request(slack_request).await
            }
            SlackRequestBody::OAuth(oauth_req) => {
                self.handle_oauth_request(slack_request.clone(), oauth_req).await
            }
            SlackRequestBody::Raw(_) => {
                Ok(SlackResponse::empty())
            }
        }
    }

    fn to_slack_request(&self, request: ApiGatewayProxyRequest) -> Result<SlackRequest> {
        let method = request.http_method;
        let path = request.path.unwrap_or_default();
        let headers = request.headers.unwrap_or_default();
        let query_params = request.query_string_parameters.unwrap_or_default();
        let body = request.body.unwrap_or_default();

        let slack_body = self.parse_body(&body, &headers)?;

        Ok(SlackRequest {
            method,
            path,
            headers,
            query_params,
            body: slack_body,
        })
    }

    fn parse_body(&self, body: &str, headers: &HashMap<String, String>) -> Result<SlackRequestBody> {
        let content_type = headers.get("content-type")
            .or_else(|| headers.get("Content-Type"))
            .unwrap_or(&"".to_string())
            .to_lowercase();

        if content_type.contains("application/json") {
            // Event API request
            let event_req: EventRequest = serde_json::from_str(body)?;
            Ok(SlackRequestBody::Event(event_req))
        } else if content_type.contains("application/x-www-form-urlencoded") {
            // Parse form data
            let form_data = self.parse_form_data(body)?;
            
            if let Some(payload) = form_data.get("payload") {
                // Interactive component request
                let interactive_req: InteractiveRequest = serde_json::from_str(payload)?;
                Ok(SlackRequestBody::Interactive(interactive_req))
            } else if form_data.contains_key("command") {
                // Slash command request
                let command_req = CommandRequest {
                    token: form_data.get("token").unwrap_or(&"".to_string()).clone(),
                    team_id: form_data.get("team_id").unwrap_or(&"".to_string()).clone(),
                    team_domain: form_data.get("team_domain").unwrap_or(&"".to_string()).clone(),
                    channel_id: form_data.get("channel_id").unwrap_or(&"".to_string()).clone(),
                    channel_name: form_data.get("channel_name").unwrap_or(&"".to_string()).clone(),
                    user_id: form_data.get("user_id").unwrap_or(&"".to_string()).clone(),
                    user_name: form_data.get("user_name").unwrap_or(&"".to_string()).clone(),
                    command: form_data.get("command").unwrap_or(&"".to_string()).clone(),
                    text: form_data.get("text").unwrap_or(&"".to_string()).clone(),
                    response_url: form_data.get("response_url").unwrap_or(&"".to_string()).clone(),
                    trigger_id: form_data.get("trigger_id").unwrap_or(&"".to_string()).clone(),
                };
                Ok(SlackRequestBody::Command(command_req))
            } else if form_data.contains_key("code") || form_data.contains_key("error") {
                // OAuth callback
                let oauth_req = OAuthRequest {
                    code: form_data.get("code").cloned(),
                    state: form_data.get("state").cloned(),
                    error: form_data.get("error").cloned(),
                };
                Ok(SlackRequestBody::OAuth(oauth_req))
            } else {
                Ok(SlackRequestBody::Raw(body.to_string()))
            }
        } else {
            Ok(SlackRequestBody::Raw(body.to_string()))
        }
    }

    fn parse_form_data(&self, body: &str) -> Result<HashMap<String, String>> {
        let mut form_data = HashMap::new();
        
        for pair in body.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let decoded_key = decode(key).map_err(|_| SlackError::Internal("Failed to decode form key".to_string()))?;
                let decoded_value = decode(value).map_err(|_| SlackError::Internal("Failed to decode form value".to_string()))?;
                form_data.insert(decoded_key.to_string(), decoded_value.to_string());
            }
        }
        
        Ok(form_data)
    }

    fn verify_signature(&self, request: &SlackRequest) -> Result<()> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let timestamp = request.headers.get("x-slack-request-timestamp")
            .ok_or(SlackError::InvalidSignature)?;
        
        let signature = request.headers.get("x-slack-signature")
            .ok_or(SlackError::InvalidSignature)?;

        let body = match &request.body {
            SlackRequestBody::Raw(raw) => raw.clone(),
            _ => serde_json::to_string(&request.body)?,
        };

        let basestring = format!("v0:{}:{}", timestamp, body);
        
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.app.config().signing_secret.as_bytes())
            .map_err(|_| SlackError::InvalidSignature)?;
        
        mac.update(basestring.as_bytes());
        let computed_signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

        if computed_signature != *signature {
            return Err(SlackError::InvalidSignature);
        }

        Ok(())
    }

    async fn handle_event_request(&self, request: SlackRequest) -> Result<SlackResponse> {
        let client = SlackClient::new(self.app.config().get_bot_token().map(|s| s.to_string()));
        let context = Context::new(request, client);
        
        // Route through the app's event router
        if let Some(response) = self.app.router().route_request(&context.request).await? {
            Ok(response)
        } else {
            Ok(SlackResponse::empty())
        }
    }

    async fn handle_command_request(&self, request: SlackRequest) -> Result<SlackResponse> {
        let client = SlackClient::new(self.app.config().get_bot_token().map(|s| s.to_string()));
        let context = Context::new(request, client);
        
        // Route through the app's command router
        if let Some(response) = self.app.router().route_request(&context.request).await? {
            Ok(response)
        } else {
            Ok(SlackResponse::empty())
        }
    }

    async fn handle_interactive_request(&self, request: SlackRequest) -> Result<SlackResponse> {
        let client = SlackClient::new(self.app.config().get_bot_token().map(|s| s.to_string()));
        let context = Context::new(request, client);
        
        // Route through the app's interactive router
        if let Some(response) = self.app.router().route_request(&context.request).await? {
            Ok(response)
        } else {
            Ok(SlackResponse::empty())
        }
    }

    async fn handle_oauth_request(&self, request: SlackRequest, oauth_req: &OAuthRequest) -> Result<SlackResponse> {
        if let Some(oauth_settings) = self.app.oauth_settings() {
            if let Some(error) = &oauth_req.error {
                error!("OAuth error: {}", error);
                return Ok(SlackResponse {
                    status_code: 400,
                    headers: HashMap::new(),
                    body: crate::response::SlackResponseBody::Text(crate::response::TextResponse {
                        text: format!("OAuth error: {}", error),
                        response_type: None,
                        replace_original: None,
                        delete_original: None,
                    }),
                });
            }

            if let (Some(code), Some(state)) = (&oauth_req.code, &oauth_req.state) {
                // Handle OAuth completion - this would need the OAuth flow implementation
                info!("OAuth callback received with code and state");
                // In a real implementation, you'd complete the OAuth flow here
                Ok(SlackResponse::text("Installation successful!"))
            } else {
                // Start OAuth flow
                info!("Starting OAuth flow");
                // In a real implementation, you'd redirect to Slack's OAuth URL
                Ok(SlackResponse::redirect("https://slack.com/oauth/v2/authorize"))
            }
        } else {
            Ok(SlackResponse {
                status_code: 404,
                headers: HashMap::new(),
                body: crate::response::SlackResponseBody::Empty,
            })
        }
    }

    fn to_api_gateway_response(&self, response: SlackResponse) -> ApiGatewayProxyResponse {
        let body = match response.body {
            crate::response::SlackResponseBody::Empty => None,
            _ => Some(serde_json::to_string(&response.body).unwrap_or_default()),
        };

        ApiGatewayProxyResponse {
            status_code: response.status_code as i32,
            headers: response.headers,
            body,
            is_base64_encoded: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiGatewayProxyRequest {
    #[serde(rename = "httpMethod")]
    http_method: String,
    path: Option<String>,
    #[serde(rename = "queryStringParameters")]
    query_string_parameters: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    #[serde(rename = "isBase64Encoded")]
    is_base64_encoded: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ApiGatewayProxyResponse {
    #[serde(rename = "statusCode")]
    status_code: i32,
    headers: HashMap<String, String>,
    body: Option<String>,
    #[serde(rename = "isBase64Encoded")]
    is_base64_encoded: bool,
}