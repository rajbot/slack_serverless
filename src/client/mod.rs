use crate::error::{Result, SlackError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SlackClient {
    client: Client,
    token: Option<String>,
    base_url: String,
}

impl SlackClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://slack.com/api".to_string(),
        }
    }

    pub async fn post_message(&self, request: &PostMessageRequest) -> Result<PostMessageResponse> {
        let url = format!("{}/chat.postMessage", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_token()?))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let response_body: PostMessageResponse = response.json().await?;
        
        if !response_body.ok {
            return Err(SlackError::SlackApi {
                code: response_body.error.clone().unwrap_or_default(),
                message: "API call failed".to_string(),
            });
        }

        Ok(response_body)
    }

    pub async fn update_message(&self, request: &UpdateMessageRequest) -> Result<UpdateMessageResponse> {
        let url = format!("{}/chat.update", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_token()?))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let response_body: UpdateMessageResponse = response.json().await?;
        
        if !response_body.ok {
            return Err(SlackError::SlackApi {
                code: response_body.error.clone().unwrap_or_default(),
                message: "API call failed".to_string(),
            });
        }

        Ok(response_body)
    }

    pub async fn delete_message(&self, request: &DeleteMessageRequest) -> Result<DeleteMessageResponse> {
        let url = format!("{}/chat.delete", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_token()?))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let response_body: DeleteMessageResponse = response.json().await?;
        
        if !response_body.ok {
            return Err(SlackError::SlackApi {
                code: response_body.error.clone().unwrap_or_default(),
                message: "API call failed".to_string(),
            });
        }

        Ok(response_body)
    }

    fn get_token(&self) -> Result<&str> {
        self.token.as_deref().ok_or_else(|| {
            SlackError::Config("Bot token is required for API calls".to_string())
        })
    }
}

#[derive(Debug, Serialize)]
pub struct PostMessageRequest {
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostMessageResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct UpdateMessageRequest {
    pub channel: String,
    pub ts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteMessageRequest {
    pub channel: String,
    pub ts: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMessageResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
}