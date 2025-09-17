use crate::error::{Result, SlackError};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bot_token: Option<String>,
    pub signing_secret: String,
    pub app_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
    pub user_scopes: Vec<String>,
}

impl AppConfig {
    pub fn new(signing_secret: String) -> Self {
        Self {
            bot_token: None,
            signing_secret,
            app_token: None,
            client_id: None,
            client_secret: None,
            redirect_uri: None,
            scopes: vec!["chat:write".to_string()],
            user_scopes: vec![],
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.signing_secret.is_empty() {
            return Err(SlackError::Config("Signing secret is required".to_string()));
        }

        if self.bot_token.is_none() && self.client_id.is_none() {
            return Err(SlackError::Config(
                "Either bot_token or client_id must be provided".to_string(),
            ));
        }

        if self.client_id.is_some() && self.client_secret.is_none() {
            return Err(SlackError::Config(
                "client_secret is required when client_id is provided".to_string(),
            ));
        }

        Ok(())
    }

    pub fn is_oauth_enabled(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }

    pub fn get_bot_token(&self) -> Option<&str> {
        self.bot_token.as_deref()
    }
}