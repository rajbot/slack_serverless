use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: SlackResponseBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlackResponseBody {
    Text(TextResponse),
    Blocks(BlocksResponse),
    Challenge(ChallengeResponse),
    OAuth(OAuthResponse),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_original: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_original: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocksResponse {
    pub blocks: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_original: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_original: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub url: String,
}

impl SlackResponse {
    pub fn empty() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: SlackResponseBody::Empty,
        }
    }

    pub fn text<S: Into<String>>(text: S) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: SlackResponseBody::Text(TextResponse {
                text: text.into(),
                response_type: None,
                replace_original: None,
                delete_original: None,
            }),
        }
    }

    pub fn challenge<S: Into<String>>(challenge: S) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: SlackResponseBody::Challenge(ChallengeResponse {
                challenge: challenge.into(),
            }),
        }
    }

    pub fn redirect<S: Into<String>>(url: S) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url.into());
        
        Self {
            status_code: 302,
            headers,
            body: SlackResponseBody::Empty,
        }
    }
}