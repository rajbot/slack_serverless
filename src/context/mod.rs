pub mod ack;

pub use ack::Ack;

use crate::client::SlackClient;
use crate::request::SlackRequest;
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub request: Arc<SlackRequest>,
    pub client: Arc<SlackClient>,
    pub ack: Ack,
    pub say: Say,
    pub body: Value,
    pub payload: Value,
    pub logger: tracing::Span,
    pub custom: HashMap<String, Value>,
}

impl Context {
    pub fn new(
        request: SlackRequest,
        client: SlackClient,
    ) -> Self {
        let request_arc = Arc::new(request);
        let client_arc = Arc::new(client);
        
        Self {
            ack: Ack::new(request_arc.clone()),
            say: Say::new(client_arc.clone(), request_arc.clone()),
            body: Value::Null,
            payload: Value::Null,
            logger: tracing::span!(tracing::Level::INFO, "slack_request"),
            custom: HashMap::new(),
            request: request_arc,
            client: client_arc,
        }
    }

    pub fn set_custom<K: Into<String>>(&mut self, key: K, value: Value) {
        self.custom.insert(key.into(), value);
    }

    pub fn get_custom<K: AsRef<str>>(&self, key: K) -> Option<&Value> {
        self.custom.get(key.as_ref())
    }
}

#[derive(Clone)]
pub struct Say {
    client: Arc<SlackClient>,
    request: Arc<SlackRequest>,
}

impl Say {
    pub fn new(client: Arc<SlackClient>, request: Arc<SlackRequest>) -> Self {
        Self { client, request }
    }

    pub async fn text<S: Into<String>>(&self, text: S) -> Result<()> {
        // Extract channel from request and send message
        // This is a placeholder implementation
        Ok(())
    }

    pub async fn blocks(&self, blocks: Vec<Value>) -> Result<()> {
        // Send message with blocks
        // This is a placeholder implementation
        Ok(())
    }
}