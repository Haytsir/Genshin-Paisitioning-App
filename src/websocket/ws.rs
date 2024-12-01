use crate::app::terminate_process;
use crate::models::RequestEvent;
use crate::{
    models::{AppEvent, RequestDataTypes},
    websocket::{Client, Clients},
};
use crossbeam_channel::Sender;
use futures::{FutureExt, StreamExt};
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(
    ws: WebSocket,
    id: String,
    clients: Clients,
    mut client: Client,
    sender: Sender<AppEvent>,
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
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log::debug!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients, sender.clone());
    }

    clients.write().await.remove(&id);
    log::debug!("{} disconnected", id);
    // 연결이 종료되어 클라이언트가 0명이라면 프로그램도 종료한다.
    if clients.write().await.len() == 0 {
        terminate_process();
    }
}

// 클라이언트로부터 메세지를 받았을 경우의 최초 처리
fn client_msg(id: &str, msg: Message, _clients: &Clients, sender: Sender<AppEvent>) {
    log::debug!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let req: RequestEvent = match from_str(message) {
        Ok(v) => v,
        Err(e) => {
            log::debug!("error while parsing message to request: {}", e);
            return;
        }
    };
    match req.event.as_str() {
        "init" => {
            log::debug!("init!");
            let res = sender.send(AppEvent::Init());
            match res {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed INIT: {e}")
                }
            }
        }
        "uninit" => {
            log::debug!("uninit!");
            let res = sender.send(AppEvent::Uninit());
            match res {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed UNINIT: {e}")
                }
            }
        }
        "getConfig" => {
            log::debug!("getConfig!");
            let _ = sender.send(AppEvent::GetConfig(id.to_string()));
        }
        "setConfig" => {
            log::debug!("setConfig!");
            let _: RequestDataTypes = match req.data {
                Some(RequestDataTypes::AppConfig(app_config)) => {
                    log::debug!("setConfig: {:?}", app_config);
                    let _ = sender.send(AppEvent::SetConfig(app_config, id.to_string()));
                    return;
                }
                Some(_) => return,
                None => {
                    log::error!("setConfig: None Data");
                    return;
                }
            };
        }
        "checkAppUpdate" => {
            log::debug!("checkAppUpdate!");
            let force = match req.data {
                Some(RequestDataTypes::CheckAppUpdate(force)) => force,
                Some(_) => false,
                None => false,
            };
            let res = sender.send(AppEvent::CheckAppUpdate(id.to_string(), force));
            match res {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed checkAppUpdate: {e}")
                }
            }
        }
        "checkLibUpdate" => {
            log::debug!("checkLibUpdate!");
            let force = match req.data {
                Some(RequestDataTypes::CheckLibUpdate(force)) => force,
                Some(_) => false,
                None => false,
            };
            let res = sender.send(AppEvent::CheckLibUpdate(id.to_string(), force));
            match res {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed checkLibUpdate: {e}")
                }
            }
        }
        _ => {}
    }
}
