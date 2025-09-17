use crate::request::SlackRequest;
use crate::response::{SlackResponse, SlackResponseBody, TextResponse, BlocksResponse};
use crate::error::Result;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct Ack {
    request: Arc<SlackRequest>,
    acknowledged: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl Ack {
    pub fn new(request: Arc<SlackRequest>) -> Self {
        Self {
            request,
            acknowledged: Arc::new(std::sync::Mutex::new(false)),
        }
    }

    pub async fn empty(&self) -> Result<SlackResponse> {
        self.mark_acknowledged();
        Ok(SlackResponse::empty())
    }

    pub async fn text<S: Into<String>>(&self, text: S) -> Result<SlackResponse> {
        self.mark_acknowledged();
        Ok(SlackResponse::text(text))
    }

    pub async fn blocks(&self, blocks: Vec<Value>) -> Result<SlackResponse> {
        self.mark_acknowledged();
        Ok(SlackResponse {
            status_code: 200,
            headers: std::collections::HashMap::new(),
            body: SlackResponseBody::Blocks(BlocksResponse {
                blocks,
                text: None,
                response_type: None,
                replace_original: None,
                delete_original: None,
            }),
        })
    }

    pub async fn ephemeral<S: Into<String>>(&self, text: S) -> Result<SlackResponse> {
        self.mark_acknowledged();
        Ok(SlackResponse {
            status_code: 200,
            headers: std::collections::HashMap::new(),
            body: SlackResponseBody::Text(TextResponse {
                text: text.into(),
                response_type: Some("ephemeral".to_string()),
                replace_original: None,
                delete_original: None,
            }),
        })
    }

    pub async fn in_channel<S: Into<String>>(&self, text: S) -> Result<SlackResponse> {
        self.mark_acknowledged();
        Ok(SlackResponse {
            status_code: 200,
            headers: std::collections::HashMap::new(),
            body: SlackResponseBody::Text(TextResponse {
                text: text.into(),
                response_type: Some("in_channel".to_string()),
                replace_original: None,
                delete_original: None,
            }),
        })
    }

    fn mark_acknowledged(&self) {
        if let Ok(mut acked) = self.acknowledged.lock() {
            *acked = true;
        }
    }

    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged.lock().map(|acked| *acked).unwrap_or(false)
    }
}