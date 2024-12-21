pub use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;

use super::{AppConfig, AppInfo, TrackData, UpdateInfo};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct SendEvent {
    pub event: String,
    pub data: Option<DataTypes>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
#[serde(untagged)]
#[derive(Debug, Clone)]
pub enum DataTypes {
    TrackData(TrackData),
    AppInfo(AppInfo),
    AppConfig(AppConfig),
    UpdateInfo(UpdateInfo),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone)]
pub enum WsEvent {
    GetConfig,
    SetConfig { config: AppConfig },
    Config { config: AppConfig, id: String },
    Init,
    DoneInit,
    Uninit,
    Track { data: TrackData },
    CheckLibUpdate { id: String },
    CheckAppUpdate { id: String },
    #[serde(rename = "update")]
    UpdateInfo { info: Option<UpdateInfo> },
}

impl From<WsEvent> for SendEvent {
    fn from(event: WsEvent) -> Self {
        let event_name = to_variant_name(&event).unwrap();
        let data = match &event {
            WsEvent::Track { data } => Some(DataTypes::TrackData(data.clone())),
            WsEvent::Config { config, id: _ } => Some(DataTypes::AppConfig(config.clone())),
            WsEvent::UpdateInfo { info } if info.is_some() => {
                Some(DataTypes::UpdateInfo(info.clone().unwrap()))
            },
            _ => None
        };
        
        SendEvent {
            event: event_name.to_string(),
            data
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone)]
pub struct RequestEvent {
    pub event: String,
    pub data: Option<RequestDataTypes>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[derive(Debug, Clone)]
pub enum RequestDataTypes {
    AppConfig(AppConfig),
    CheckAppUpdate(RequestUpdateCheck),
    CheckLibUpdate(RequestUpdateCheck),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone)]
pub struct RequestUpdateCheck {
    pub force: bool,
}