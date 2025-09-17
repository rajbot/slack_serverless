use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub team_id: String,
    pub enterprise_id: Option<String>,
    pub bot_token: Option<String>,
    pub bot_user_id: Option<String>,
    pub user_token: Option<String>,
    pub user_id: Option<String>,
    pub scopes: Vec<String>,
    pub user_scopes: Vec<String>,
    pub installed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Installation {
    pub fn new(team_id: String) -> Self {
        Self {
            team_id,
            enterprise_id: None,
            bot_token: None,
            bot_user_id: None,
            user_token: None,
            user_id: None,
            scopes: Vec::new(),
            user_scopes: Vec::new(),
            installed_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn with_bot_token<S: Into<String>>(mut self, token: S, user_id: S) -> Self {
        self.bot_token = Some(token.into());
        self.bot_user_id = Some(user_id.into());
        self
    }

    pub fn with_user_token<S: Into<String>>(mut self, token: S, user_id: S) -> Self {
        self.user_token = Some(token.into());
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_user_scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.user_scopes = scopes.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_enterprise_id<S: Into<String>>(mut self, enterprise_id: S) -> Self {
        self.enterprise_id = Some(enterprise_id.into());
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }
}

#[async_trait]
pub trait InstallationStore: Send + Sync + Debug {
    async fn save(&self, installation: &Installation) -> Result<()>;
    
    async fn find_by_team(&self, team_id: &str, enterprise_id: Option<&str>) -> Result<Option<Installation>>;
    
    async fn delete(&self, team_id: &str, enterprise_id: Option<&str>) -> Result<()>;
    
    async fn find_bot_token(&self, team_id: &str, enterprise_id: Option<&str>) -> Result<Option<String>> {
        let installation = self.find_by_team(team_id, enterprise_id).await?;
        Ok(installation.and_then(|i| i.bot_token))
    }
    
    async fn find_user_token(&self, team_id: &str, user_id: &str, enterprise_id: Option<&str>) -> Result<Option<String>> {
        let installation = self.find_by_team(team_id, enterprise_id).await?;
        Ok(installation.and_then(|i| {
            if i.user_id.as_deref() == Some(user_id) {
                i.user_token
            } else {
                None
            }
        }))
    }
}