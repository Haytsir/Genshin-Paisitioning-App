#[allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::error::Error;
use crate::models::AppEvent;
type EventHandler = Box<dyn Fn(&AppEvent) -> Result<(), Box<dyn Error>> + Send + Sync>;

pub struct EventBus {
    handlers: Arc<RwLock<HashMap<AppEvent, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn emit(&self, event: &AppEvent) -> Result<(), Box<dyn Error>> {
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(event) {
            for handler in event_handlers {
                if let Err(e) = handler(event) { log::error!("Error handling event: {}", e); }
            }
        }
        Ok(())
    }
} 