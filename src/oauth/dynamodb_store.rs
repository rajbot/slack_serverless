#[cfg(feature = "oauth")]
use crate::error::{Result, SlackError};
use crate::oauth::{InstallationStore, StateStore, Installation, OAuthState};
use async_trait::async_trait;
use aws_sdk_dynamodb::{Client as DynamoDbClient, types::AttributeValue};
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DynamoDbInstallationStore {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoDbInstallationStore {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn create_table(&self) -> Result<()> {
        let key_schema = vec![
            aws_sdk_dynamodb::types::KeySchemaElement::builder()
                .attribute_name("team_id")
                .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
            aws_sdk_dynamodb::types::KeySchemaElement::builder()
                .attribute_name("enterprise_id")
                .key_type(aws_sdk_dynamodb::types::KeyType::Range)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
        ];

        let attribute_definitions = vec![
            aws_sdk_dynamodb::types::AttributeDefinition::builder()
                .attribute_name("team_id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
            aws_sdk_dynamodb::types::AttributeDefinition::builder()
                .attribute_name("enterprise_id")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
        ];

        self.client
            .create_table()
            .table_name(&self.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(attribute_definitions))
            .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    fn installation_to_item(&self, installation: &Installation) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        
        item.insert("team_id".to_string(), AttributeValue::S(installation.team_id.clone()));
        item.insert("enterprise_id".to_string(), AttributeValue::S(
            installation.enterprise_id.clone().unwrap_or_else(|| "NONE".to_string())
        ));
        
        if let Some(bot_token) = &installation.bot_token {
            item.insert("bot_token".to_string(), AttributeValue::S(bot_token.clone()));
        }
        
        if let Some(bot_user_id) = &installation.bot_user_id {
            item.insert("bot_user_id".to_string(), AttributeValue::S(bot_user_id.clone()));
        }
        
        if let Some(user_token) = &installation.user_token {
            item.insert("user_token".to_string(), AttributeValue::S(user_token.clone()));
        }
        
        if let Some(user_id) = &installation.user_id {
            item.insert("user_id".to_string(), AttributeValue::S(user_id.clone()));
        }
        
        item.insert("scopes".to_string(), AttributeValue::S(installation.scopes.join(",")));
        item.insert("user_scopes".to_string(), AttributeValue::S(installation.user_scopes.join(",")));
        item.insert("installed_at".to_string(), AttributeValue::S(installation.installed_at.to_rfc3339()));
        
        if let Some(expires_at) = installation.expires_at {
            item.insert("expires_at".to_string(), AttributeValue::S(expires_at.to_rfc3339()));
        }
        
        item
    }

    fn item_to_installation(&self, item: HashMap<String, AttributeValue>) -> Result<Installation> {
        let team_id = item.get("team_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| SlackError::Internal("Missing team_id".to_string()))?
            .clone();

        let enterprise_id = item.get("enterprise_id")
            .and_then(|v| v.as_s().ok())
            .filter(|s| *s != "NONE")
            .map(|s| s.clone());

        let bot_token = item.get("bot_token").and_then(|v| v.as_s().ok()).map(|s| s.clone());
        let bot_user_id = item.get("bot_user_id").and_then(|v| v.as_s().ok()).map(|s| s.clone());
        let user_token = item.get("user_token").and_then(|v| v.as_s().ok()).map(|s| s.clone());
        let user_id = item.get("user_id").and_then(|v| v.as_s().ok()).map(|s| s.clone());

        let scopes = item.get("scopes")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let user_scopes = item.get("user_scopes")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let installed_at = item.get("installed_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let expires_at = item.get("expires_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Installation {
            team_id,
            enterprise_id,
            bot_token,
            bot_user_id,
            user_token,
            user_id,
            scopes,
            user_scopes,
            installed_at,
            expires_at,
        })
    }
}

#[async_trait]
impl InstallationStore for DynamoDbInstallationStore {
    async fn save(&self, installation: &Installation) -> Result<()> {
        let item = self.installation_to_item(installation);
        
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn find_by_team(&self, team_id: &str, enterprise_id: Option<&str>) -> Result<Option<Installation>> {
        let enterprise_key = enterprise_id.unwrap_or("NONE");
        
        let response = self.client
            .get_item()
            .table_name(&self.table_name)
            .key("team_id", AttributeValue::S(team_id.to_string()))
            .key("enterprise_id", AttributeValue::S(enterprise_key.to_string()))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        if let Some(item) = response.item {
            Ok(Some(self.item_to_installation(item)?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, team_id: &str, enterprise_id: Option<&str>) -> Result<()> {
        let enterprise_key = enterprise_id.unwrap_or("NONE");
        
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("team_id", AttributeValue::S(team_id.to_string()))
            .key("enterprise_id", AttributeValue::S(enterprise_key.to_string()))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DynamoDbStateStore {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoDbStateStore {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn create_table(&self) -> Result<()> {
        let key_schema = vec![
            aws_sdk_dynamodb::types::KeySchemaElement::builder()
                .attribute_name("state")
                .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
        ];

        let attribute_definitions = vec![
            aws_sdk_dynamodb::types::AttributeDefinition::builder()
                .attribute_name("state")
                .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                .build()
                .map_err(|e| SlackError::DynamoDb(e.to_string()))?,
        ];

        self.client
            .create_table()
            .table_name(&self.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(attribute_definitions))
            .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl StateStore for DynamoDbStateStore {
    async fn save(&self, state: &OAuthState) -> Result<()> {
        let mut item = HashMap::new();
        item.insert("state".to_string(), AttributeValue::S(state.state.clone()));
        item.insert("created_at".to_string(), AttributeValue::S(state.created_at.to_rfc3339()));
        item.insert("expires_at".to_string(), AttributeValue::S(state.expires_at.to_rfc3339()));
        
        if let Some(redirect_uri) = &state.redirect_uri {
            item.insert("redirect_uri".to_string(), AttributeValue::S(redirect_uri.clone()));
        }
        
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn find(&self, state: &str) -> Result<Option<OAuthState>> {
        let response = self.client
            .get_item()
            .table_name(&self.table_name)
            .key("state", AttributeValue::S(state.to_string()))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        if let Some(item) = response.item {
            let state_value = item.get("state")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| SlackError::Internal("Missing state".to_string()))?
                .clone();

            let created_at = item.get("created_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let expires_at = item.get("expires_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now() + chrono::Duration::minutes(10));

            let redirect_uri = item.get("redirect_uri")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.clone());

            Ok(Some(OAuthState {
                state: state_value,
                redirect_uri,
                created_at,
                expires_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, state: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("state", AttributeValue::S(state.to_string()))
            .send()
            .await
            .map_err(|e| SlackError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        // In a real implementation, you'd use a scan with a filter expression
        // For now, return 0 as this is a basic implementation
        Ok(0)
    }
}