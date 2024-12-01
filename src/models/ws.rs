pub use serde::{Deserialize, Serialize};

use super::{AppConfig, AppInfo, TrackData, UpdateInfo};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct SendEvent {
    pub event: WsEvent,
    pub data: DataTypes,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
#[serde(untagged)]
#[derive(Debug, Clone)]
pub enum DataTypes {
    TrackData(TrackData),
    AppInfo(AppInfo),
    UpdateInfo(UpdateInfo, String),
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
#[serde(untagged)]
#[derive(Debug, Clone)]
pub enum WsEvent {
    GetConfig(),
    SetConfig(AppConfig),
    Config(AppConfig, String),
    Init(),
    Uninit(),
    Track(TrackData),
    CheckLibUpdate(String),
    CheckAppUpdate(String),
    UpdateInfo(UpdateInfo, String),
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
    CheckAppUpdate(bool),
    CheckLibUpdate(bool),
}
