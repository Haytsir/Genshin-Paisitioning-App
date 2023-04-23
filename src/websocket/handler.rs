use crate::{
    models::{AppConfig, AppEvent, UpdateInfo},
    websocket::{ws, Client, Clients, Result},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

use crate::models::TrackData;
use crossbeam_channel::Sender;

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

pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
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

// TODO: app info send 함수, data를 받아오는 부분 작성해야 함
/* pub async fn send_app_info(client: &Client) -> Result<impl Reply> {
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!("{{\"event\": \"app_info\", \"data\": {}}}", serde_json::to_string(&data).unwrap()))));
    }
    Ok(StatusCode::OK)
} */

pub async fn send_config(data: AppConfig, clients: Clients, id: String) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned().unwrap();
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!(
            "{{\"event\": \"config\", \"data\": {}}}",
            serde_json::to_string(&data).unwrap()
        ))));
    }
    Ok(StatusCode::OK)
}

pub async fn send_update_info(
    data: UpdateInfo,
    clients: Clients,
    id: String,
) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned().unwrap();
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(format!(
            "{{\"event\": \"update\", \"data\": {}}}",
            serde_json::to_string(&data).unwrap()
        ))));
    }
    Ok(StatusCode::OK)
}

pub async fn broadcast_track(data: TrackData, clients: Clients) -> Result<impl Reply> {
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

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply> {
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

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.write().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    id: String,
    clients: Clients,
    sender: Sender<AppEvent>,
) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| {
            ws::client_connection(socket, id, clients, c, sender)
        })),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
