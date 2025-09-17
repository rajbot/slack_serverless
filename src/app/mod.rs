pub mod builder;
pub mod config;

pub use builder::AppBuilder;
pub use config::AppConfig;

use crate::error::{Result, SlackError};
use crate::listener::EventRouter;
use crate::middleware::MiddlewareStack;
use crate::oauth::OAuthSettings;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    config: Arc<AppConfig>,
    router: Arc<EventRouter>,
    middleware: Arc<MiddlewareStack>,
    oauth_settings: Option<Arc<OAuthSettings>>,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            router: Arc::new(EventRouter::new()),
            middleware: Arc::new(MiddlewareStack::new()),
            oauth_settings: None,
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn router(&self) -> &EventRouter {
        &self.router
    }

    pub fn middleware(&self) -> &MiddlewareStack {
        &self.middleware
    }

    pub fn oauth_settings(&self) -> Option<&OAuthSettings> {
        self.oauth_settings.as_deref()
    }

    #[cfg(feature = "lambda")]
    pub fn lambda_handler(self) -> crate::adapter::aws_lambda::LambdaHandler {
        crate::adapter::aws_lambda::LambdaHandler::new(self)
    }
}