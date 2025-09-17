use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub redirect_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl OAuthState {
    pub fn new() -> Self {
        let state = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(10); // 10 minute expiration
        
        Self {
            state,
            redirect_uri: None,
            created_at: now,
            expires_at,
        }
    }

    pub fn with_redirect_uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    pub fn is_valid(&self, state: &str) -> bool {
        !self.is_expired() && self.state == state
    }
}

#[async_trait]
pub trait StateStore: Send + Sync + Debug {
    async fn save(&self, state: &OAuthState) -> Result<()>;
    
    async fn find(&self, state: &str) -> Result<Option<OAuthState>>;
    
    async fn delete(&self, state: &str) -> Result<()>;
    
    async fn cleanup_expired(&self) -> Result<u64> {
        // Default implementation - stores can override for efficiency
        Ok(0)
    }
    
    async fn verify_and_consume(&self, state: &str) -> Result<Option<OAuthState>> {
        if let Some(oauth_state) = self.find(state).await? {
            if oauth_state.is_valid(state) {
                self.delete(state).await?;
                Ok(Some(oauth_state))
            } else {
                self.delete(state).await?; // Clean up expired state
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}