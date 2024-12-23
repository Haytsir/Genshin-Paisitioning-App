#[allow(dead_code)]
use std::collections::HashMap;
use std::{future::Future, sync::Arc};
use tokio::sync::RwLock;
use std::error::Error;
use crate::models::AppEvent;
type EventHandler = Box<dyn Fn(&AppEvent) -> futures::future::BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>> + Send + Sync>;

pub struct EventBus {
    handlers: Arc<RwLock<HashMap<AppEvent, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn emit(&self, event: &AppEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(event) {
            for handler in event_handlers {
                if let Err(e) = handler(event).await { 
                    log::error!("Error handling event: {}", e); 
                }
            }
        }
        Ok(())
    }

    pub async fn register<F, Fut>(&self, event: AppEvent, handler: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(&AppEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event)
            .or_insert_with(Vec::new)
            .push(Box::new(move |event| Box::pin(handler(event))));
        Ok(())
    }
} 