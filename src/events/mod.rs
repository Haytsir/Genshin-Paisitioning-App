use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::error::Error;
use crate::models::{AppEvent, WsEvent};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum EventType {
    Init,
    Uninit,
    GetConfig(String),
    SetConfig(String),
    CheckAppUpdate(String, bool),
    CheckLibUpdate(String, bool),
    Track,
}

impl From<AppEvent> for EventType {
    fn from(event: AppEvent) -> Self {
        match event {
            AppEvent::Init() => EventType::Init,
            AppEvent::Uninit() => EventType::Uninit,
            AppEvent::GetConfig(id) => EventType::GetConfig(id),
            AppEvent::SetConfig(_, id) => EventType::SetConfig(id),
            AppEvent::CheckAppUpdate(id, force) => EventType::CheckAppUpdate(id, force),
            AppEvent::CheckLibUpdate(id, force) => EventType::CheckLibUpdate(id, force),
            _ => EventType::Track, // 기본값 처리
        }
    }
}

type EventHandler = Box<dyn Fn(EventType) -> Result<(), Box<dyn Error>> + Send + Sync>;

pub struct EventBus {
    handlers: Arc<RwLock<HashMap<EventType, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn emit(&self, event: EventType) -> Result<(), Box<dyn Error>> {
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(&event) {
            for handler in event_handlers {
                handler(event.clone())?;
            }
        }
        Ok(())
    }
} 