pub mod flow;
pub mod installation_store;
pub mod state_store;

#[cfg(feature = "oauth")]
pub mod dynamodb_store;

pub use installation_store::{InstallationStore, Installation};
pub use state_store::{StateStore, OAuthState};

use crate::error::Result;

#[derive(Debug)]
pub struct OAuthSettings {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
    pub user_scopes: Vec<String>,
    pub installation_store: Option<Box<dyn InstallationStore>>,
    pub state_store: Option<Box<dyn StateStore>>,
}

impl OAuthSettings {
    pub fn new() -> Self {
        Self {
            client_id: None,
            client_secret: None,
            redirect_uri: None,
            scopes: vec!["chat:write".to_string()],
            user_scopes: vec![],
            installation_store: None,
            state_store: None,
        }
    }

    pub fn client_id<S: Into<String>>(mut self, client_id: S) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn client_secret<S: Into<String>>(mut self, client_secret: S) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn redirect_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    pub fn scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn user_scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.user_scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn installation_store<S: InstallationStore + 'static>(mut self, store: S) -> Self {
        self.installation_store = Some(Box::new(store));
        self
    }

    pub fn state_store<S: StateStore + 'static>(mut self, store: S) -> Self {
        self.state_store = Some(Box::new(store));
        self
    }
}