use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: SlackRequestBody,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SlackRequestBody {
    Event(EventRequest),
    Command(CommandRequest),
    Interactive(InteractiveRequest),
    OAuth(OAuthRequest),
    Raw(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventRequest {
    pub token: String,
    pub team_id: String,
    pub api_app_id: String,
    pub event: serde_json::Value,
    pub event_type: String,
    pub event_time: u64,
    pub challenge: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandRequest {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InteractiveRequest {
    pub token: String,
    pub team: serde_json::Value,
    pub user: serde_json::Value,
    pub channel: Option<serde_json::Value>,
    pub message: Option<serde_json::Value>,
    pub actions: Vec<serde_json::Value>,
    pub callback_id: Option<String>,
    pub trigger_id: String,
    pub response_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthRequest {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}