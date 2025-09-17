use crate::error::{Result, SlackError};
use crate::oauth::{InstallationStore, StateStore, Installation, OAuthState};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

pub struct OAuthFlow {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scopes: Vec<String>,
    user_scopes: Vec<String>,
    installation_store: Box<dyn InstallationStore>,
    state_store: Box<dyn StateStore>,
    http_client: Client,
}

impl OAuthFlow {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        scopes: Vec<String>,
        user_scopes: Vec<String>,
        installation_store: Box<dyn InstallationStore>,
        state_store: Box<dyn StateStore>,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            scopes,
            user_scopes,
            installation_store,
            state_store,
            http_client: Client::new(),
        }
    }

    pub async fn start(&self) -> Result<String> {
        let state = OAuthState::new().with_redirect_uri(&self.redirect_uri);
        self.state_store.save(&state).await?;

        let mut url = Url::parse("https://slack.com/oauth/v2/authorize")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("scope", &self.scopes.join(","))
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("state", &state.state);

        if !self.user_scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("user_scope", &self.user_scopes.join(","));
        }

        Ok(url.to_string())
    }

    pub async fn complete(&self, code: &str, state: &str) -> Result<Installation> {
        // Verify state
        let oauth_state = self.state_store.verify_and_consume(state).await?
            .ok_or_else(|| SlackError::OAuth("Invalid or expired state".to_string()))?;

        // Exchange code for tokens
        let token_response = self.exchange_code(code).await?;

        // Create installation
        let mut installation = Installation::new(token_response.team.id.clone())
            .with_scopes(self.scopes.clone());

        if let Some(bot) = token_response.access_token {
            installation = installation.with_bot_token(bot, token_response.bot_user_id.unwrap_or_default());
        }

        if let Some(user_token) = token_response.authed_user.access_token {
            installation = installation.with_user_token(user_token, token_response.authed_user.id);
        }

        if let Some(enterprise) = token_response.enterprise {
            installation = installation.with_enterprise_id(enterprise.id);
        }

        // Save installation
        self.installation_store.save(&installation).await?;

        Ok(installation)
    }

    async fn exchange_code(&self, code: &str) -> Result<OAuthAccessResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        let response = self.http_client
            .post("https://slack.com/api/oauth.v2.access")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        let oauth_response: OAuthAccessResponse = response.json().await?;

        if !oauth_response.ok {
            return Err(SlackError::OAuth(
                oauth_response.error.unwrap_or_else(|| "Unknown OAuth error".to_string())
            ));
        }

        Ok(oauth_response)
    }
}

#[derive(Debug, Deserialize)]
struct OAuthAccessResponse {
    ok: bool,
    error: Option<String>,
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    bot_user_id: Option<String>,
    app_id: String,
    team: Team,
    enterprise: Option<Enterprise>,
    authed_user: AuthedUser,
}

#[derive(Debug, Deserialize)]
struct Team {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Enterprise {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AuthedUser {
    id: String,
    scope: Option<String>,
    access_token: Option<String>,
    token_type: Option<String>,
}