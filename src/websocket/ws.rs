use crate::{models::{RequestEvent, SendEvent}, 
    websocket::{Client, Clients}}
;
use futures::{FutureExt, StreamExt, Future};
use serde_json::from_str;
use std::error::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::WebSocketError;

type MessageHandler = Box<
    dyn Fn(String, RequestEvent) -> futures::future::BoxFuture<'static, Result<(), Box<dyn Error + Send + Sync>>> 
    + Send 
    + Sync
>;

#[derive(Clone)]
pub struct WebSocketHandler {
    handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    pub clients: Clients,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<F, Fut>(&self, event: &str, handler: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(String, RequestEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        log::debug!("이벤트 등록 시작: {}", event);
        {
            let mut handlers = self.handlers.write().await;
            handlers.insert(event.to_string(), Box::new(move |id, req| {
                Box::pin(handler(id.to_string(), req))
            }));
        }        
        Ok(())
    }

    pub async fn handle_message(&self, id: &str, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = msg.to_str()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message format"))?;
        log::debug!("handle_message from {} : {}", id, message);

        let req: RequestEvent = match from_str(message) {
            Ok(v) => v,
            Err(e) => {
                log::debug!("error while parsing message to request: {}", e);
                return Ok(());
            }
        };
        log::debug!("event: {}", req.event);
        log::debug!("data: {:#?}", req.data);

        let handlers = self.handlers.read().await;

        if let Some(handler) = handlers.get(&req.event) {
            log::debug!("Found handler for event: {}", req.event);
            handler(id.to_string(), req).await
        } else {
            log::debug!("No handler found for event: {}", req.event);
            Ok(())
        }
    }

    pub async fn send_to(&self, client_id: String, event: SendEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::debug!("send event: {:?}", event);
        if let Some(client) = self.clients.read().await.get(&client_id) {
            if let Some(sender) = &client.sender {
                let message = Message::text(serde_json::to_string(&event)?);
                sender.send(Ok(message))
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            }
        }
        Ok(())
    }

    pub async fn broadcast(&self, event: SendEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::debug!("broadcast event: {:?}", event);
        let message = Message::text(serde_json::to_string(&event)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?);
        log::debug!("message: {:?}", message);
        for (_, client) in self.clients.read().await.iter() {
            if let Some(sender) = &client.sender {
                sender.send(Ok(message.clone()))
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            }
        }
        Ok(())
    }

    pub async fn broadcast_to(&self, client_ids: Vec<String>, event: SendEvent) -> Result<(), warp::Rejection> {
        let message = Message::text(serde_json::to_string(&event)
                    .map_err(|e| warp::reject::custom(WebSocketError(e.to_string())))?);
        for client_id in client_ids {
            if let Some(sender) = self.clients.read().await.get(&client_id).and_then(|c| c.sender.as_ref()) {
                sender.send(Ok(message.clone()))
                    .map_err(|e| warp::reject::custom(WebSocketError(e.to_string())))?;
            }
        }
        Ok(())
    }
}

pub async fn client_connection(
    ws: WebSocket,
    id: String,
    clients: Clients,
    mut client: Client,
    ws_handler: Arc<WebSocketHandler>,
) {    
    let (sink, mut stream) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(sink).map(|result| {
        if let Err(e) = result {
            log::debug!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(tx);
    clients.write().await.insert(id.clone(), client);

    log::debug!("{} connected", id);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Err(e) = ws_handler.handle_message(&id, msg).await {
                    log::error!("Error handling message: {}", e);
                    break;
                }
            }
            Err(e) => {
                log::debug!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
    }

    // 클라이언트 연결이 종료되면 안전하게 제거
    let mut clients_guard = clients.write().await;
    clients_guard.remove(&id);
    log::debug!("{} disconnected", id);
    
    #[cfg(not(debug_assertions))]
    if clients_guard.is_empty() {
        terminate_process();
    }
}