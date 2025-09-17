use crate::app::{App, AppConfig};
use crate::error::{Result, SlackError};
use crate::oauth::OAuthSettings;
use std::env;
use std::sync::Arc;

pub struct AppBuilder {
    config: AppConfig,
    oauth_settings: Option<OAuthSettings>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::new(String::new()),
            oauth_settings: None,
        }
    }

    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.config.bot_token = Some(token.into());
        self
    }

    pub fn token_from_env<S: AsRef<str>>(mut self, env_var: S) -> Result<Self> {
        let token = env::var(env_var.as_ref())
            .map_err(|_| SlackError::MissingEnvVar(env_var.as_ref().to_string()))?;
        self.config.bot_token = Some(token);
        Ok(self)
    }

    pub fn signing_secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.config.signing_secret = secret.into();
        self
    }

    pub fn signing_secret_from_env<S: AsRef<str>>(mut self, env_var: S) -> Result<Self> {
        let secret = env::var(env_var.as_ref())
            .map_err(|_| SlackError::MissingEnvVar(env_var.as_ref().to_string()))?;
        self.config.signing_secret = secret;
        Ok(self)
    }

    pub fn client_id<S: Into<String>>(mut self, client_id: S) -> Self {
        self.config.client_id = Some(client_id.into());
        self
    }

    pub fn client_id_from_env<S: AsRef<str>>(mut self, env_var: S) -> Result<Self> {
        let client_id = env::var(env_var.as_ref())
            .map_err(|_| SlackError::MissingEnvVar(env_var.as_ref().to_string()))?;
        self.config.client_id = Some(client_id);
        Ok(self)
    }

    pub fn client_secret<S: Into<String>>(mut self, client_secret: S) -> Self {
        self.config.client_secret = Some(client_secret.into());
        self
    }

    pub fn client_secret_from_env<S: AsRef<str>>(mut self, env_var: S) -> Result<Self> {
        let client_secret = env::var(env_var.as_ref())
            .map_err(|_| SlackError::MissingEnvVar(env_var.as_ref().to_string()))?;
        self.config.client_secret = Some(client_secret);
        Ok(self)
    }

    pub fn scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.config.scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn user_scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.config.user_scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn redirect_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.config.redirect_uri = Some(uri.into());
        self
    }

    pub fn oauth_settings<F>(mut self, f: F) -> Self
    where
        F: FnOnce(OAuthSettings) -> OAuthSettings,
    {
        let settings = OAuthSettings::new();
        self.oauth_settings = Some(f(settings));
        self
    }

    pub fn build(self) -> Result<App> {
        self.config.validate()?;

        let mut app = App::new(self.config);
        
        if let Some(oauth_settings) = self.oauth_settings {
            app.oauth_settings = Some(Arc::new(oauth_settings));
        }

        Ok(app)
    }
}