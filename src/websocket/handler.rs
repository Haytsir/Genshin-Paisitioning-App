use std::sync::Arc;

use crate::models::{AppConfig, UpdateInfo};

use crate::websocket::{Client, Clients};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

use crate::models::TrackData;

use super::ws::client_connection;
use super::WebSocketHandler;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    //    topic: String,
    user_id: Option<usize>,
    message: String,
}

pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply, warp::Rejection> {
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| match body.user_id {
            Some(v) => client.user_id == v,
            None => true,
        })
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        });

    Ok(StatusCode::OK)
}

pub async fn send_config(config: AppConfig, clients: Clients, id: String) -> Result<impl Reply, warp::Rejection> {
    let client = clients.read().await.get(&id).cloned().unwrap();
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!(
            "{{\"event\": \"config\", \"data\": {}}}",
            serde_json::to_string(&config).unwrap()
        ))));
    }
    Ok(StatusCode::OK)
}

pub async fn send_update_info(
    data: UpdateInfo,
    clients: Clients,
    id: String,
) -> Result<impl Reply, warp::Rejection> {
    let client = clients.read().await.get(&id).cloned().unwrap();
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!(
            "{{\"event\": \"update\", \"data\": {}}}",
            serde_json::to_string(&data).unwrap()
        ))));
    }
    Ok(StatusCode::OK)
}

pub async fn broadcast_init(clients: Clients) -> Result<impl Reply, warp::Rejection> {
    log::debug!("Broadcast Init");
    clients.read().await.iter().for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!(
            "{{\"event\": \"ready\"}}",
            ))));
        }
    });
    Ok(StatusCode::OK)
}


pub async fn broadcast_track(data: TrackData, clients: Clients) -> Result<impl Reply, warp::Rejection> {
    clients.read().await.iter().for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
            let _ = sender.send(Ok(Message::text(format!(
                "{{\"event\": \"track\", \"data\": {}}}",
                serde_json::to_string(&data).unwrap()
            ))));
        }
    });

    Ok(StatusCode::OK)
}

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply, warp::Rejection> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), user_id, clients).await;

    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:32332/ws/{}", uuid),
    }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            user_id,
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, warp::Rejection> {
    clients.write().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    id: String,
    clients: Clients,
    ws_handler: Arc<WebSocketHandler>,
) -> Result<impl Reply, warp::Rejection> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| {
            client_connection(socket, id, clients, c, ws_handler)
        })),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<impl Reply, warp::Rejection> {
    Ok(StatusCode::OK)
}

#[derive(Debug)]
pub enum CustomError {
    UPDATE_FAILED,
    CONFIG_ERROR,
}

impl warp::reject::Reject for CustomError {}