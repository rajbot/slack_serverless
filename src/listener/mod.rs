pub mod event;
pub mod command;
pub mod action;
pub mod shortcut;
pub mod message;

use crate::error::Result;
use crate::request::SlackRequest;
use crate::response::SlackResponse;
use crate::context::Context;
use std::collections::HashMap;
use std::sync::Arc;

pub type ListenerHandler = Arc<dyn Fn(Context) -> Result<SlackResponse> + Send + Sync>;

pub struct EventRouter {
    event_handlers: HashMap<String, Vec<ListenerHandler>>,
    command_handlers: HashMap<String, Vec<ListenerHandler>>,
    action_handlers: HashMap<String, Vec<ListenerHandler>>,
    shortcut_handlers: HashMap<String, Vec<ListenerHandler>>,
    message_handlers: Vec<ListenerHandler>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            event_handlers: HashMap::new(),
            command_handlers: HashMap::new(),
            action_handlers: HashMap::new(),
            shortcut_handlers: HashMap::new(),
            message_handlers: Vec::new(),
        }
    }

    pub fn add_event_handler<S: Into<String>>(&mut self, event_type: S, handler: ListenerHandler) {
        self.event_handlers
            .entry(event_type.into())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn add_command_handler<S: Into<String>>(&mut self, command: S, handler: ListenerHandler) {
        self.command_handlers
            .entry(command.into())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn add_action_handler<S: Into<String>>(&mut self, action_id: S, handler: ListenerHandler) {
        self.action_handlers
            .entry(action_id.into())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn add_shortcut_handler<S: Into<String>>(&mut self, callback_id: S, handler: ListenerHandler) {
        self.shortcut_handlers
            .entry(callback_id.into())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn add_message_handler(&mut self, handler: ListenerHandler) {
        self.message_handlers.push(handler);
    }

    pub async fn route_request(&self, request: &SlackRequest) -> Result<Option<SlackResponse>> {
        // Route based on request type
        // This is a placeholder implementation
        Ok(None)
    }
}