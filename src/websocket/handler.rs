use std::fmt::{self, Display, Formatter};

use crate::{cvat::{on_config_changed, set_capture_delay_on_error, set_capture_interval, set_is_tracking, start_track_thead}, models::{AppConfig, AppEvent, WsEvent, UpdateInfo}};
use config::Config;
use crossbeam_channel::{Receiver, Sender};
use log::debug;
use warp::reject::Reject;

use crate::websocket::{ws, Client, Clients, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

use crate::models::TrackData;

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

#[derive(Debug)]
struct CustomError {
    message: &'static str,
}

impl Reject for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl CustomError {
    const UPDATE_FAILED: CustomError = CustomError { message: "Updater: 업데이트 실패" };
}

pub fn ws_event_handler(mut config: config::Config, tx: Option<Sender<WsEvent>>, rx: Option<Receiver<AppEvent>>) -> std::result::Result<(), warp::Rejection> {
    log::info!("Start Listening Websocket Event");
    while let Some(r) = rx.as_ref() {
        let res = r.recv();
        match res {
            Ok(AppEvent::CheckAppUpdate(id, force)) => {
                debug!("Got CheckAppUpdate in Event Handler");
                let result = crate::app::updater::check_app_update(config.clone(), id.clone(), tx.clone(), force);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Updater: 업데이트 실패");
                        return Err(warp::reject::custom(CustomError::UPDATE_FAILED));
                    }
                }
            } 
            Ok(AppEvent::CheckLibUpdate(id, force)) => {
                debug!("Got CheckLibUpdate in Event Handler");
                let result = crate::app::updater::check_lib_update(config.clone(), id.clone(), tx.clone(), force);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Updater: 업데이트 실패");
                        return Err(warp::reject::custom(CustomError::UPDATE_FAILED));
                    }
                }
            }
            Ok(AppEvent::Init()) => {
                debug!("Got Init in Event Handler");
                let app_config: AppConfig = config.clone().try_deserialize().unwrap();
                set_capture_interval(u64::from(app_config.capture_interval));
                set_capture_delay_on_error(u64::from(app_config.capture_delay_on_error));
                log::debug!("Got Init");
                start_track_thead(tx.clone(), app_config.use_bit_blt_capture_mode);
            }
            Ok(AppEvent::Uninit()) => {
                log::debug!("Got Uninit in Event Handler");
                set_is_tracking(false);
                // stop_track_thread(cvat/*tx.clone()*/);
            }
            Ok(AppEvent::GetConfig(id)) => {
                log::debug!("Got GetConfig in Event Handler");
                if let Some(t) = tx.as_ref() {
                    let app_config: AppConfig = config.clone().try_deserialize().unwrap();
                    t.send(WsEvent::Config(app_config, id)).unwrap();
                }
            }
            Ok(AppEvent::SetConfig(mut new_app_config, id)) => {
                log::debug!("Got SetConfig in Event Handler");
                if let Some(t) = tx.as_ref() {
                    let new_config = Config::builder()
                        .add_source(config.clone())
                        .set_override("captureInterval", new_app_config.capture_interval)
                        .expect("Failed to set override")
                        .set_override("captureDelayOnError", new_app_config.capture_delay_on_error)
                        .expect("Failed to set override")
                        .set_override(
                            "useBitBltCaptureMode",
                            new_app_config.use_bit_blt_capture_mode,
                        )
                        .expect("Failed to set override")
                        .build();
                    
                    config = match new_config {
                        Ok(cfg) => {
                            on_config_changed(config.clone(), cfg.clone());
                            cfg
                        },
                        Err(_) => return Err(warp::reject::custom(CustomError::UPDATE_FAILED)),
                    };
                    new_app_config.changed = Some(true);
                    t.send(WsEvent::Config(new_app_config, id)).unwrap();
                }
            }
            Ok(_) => {
                log::error!("Unknown: {:#?}", res);
            }
            Err(e) => {
                log::error!("Updater: 업데이트 실패");
                return Err(warp::reject::custom(CustomError::UPDATE_FAILED));
            } //panic!("panic happened"),
        }
    }
    Ok(())
}
