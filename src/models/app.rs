pub use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: &'static str,
    pub lib_version: &'static str,
    pub configs: AppConfig,
}

impl Clone for AppInfo {
    fn clone(&self) -> AppInfo {
        *self
    }
}

#[derive(Serialize, Deserialize, Debug, Copy)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub auto_app_update: bool,
    pub auto_lib_update: bool,
    pub capture_interval: u32,
    pub capture_delay_on_error: u32,
    pub use_bit_blt_capture_mode: bool,
    pub changed: Option<bool>,
}
impl Clone for AppConfig {
    fn clone(&self) -> AppConfig {
        *self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub target_type: String,
    pub target_version: String,
    pub current_version: String,
    pub downloaded: u64,
    pub file_size: u64,
    pub percent: f64,
    pub done: bool,
    pub updated: bool
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    Init(),
    Uninit(),
    GetConfig(String),
    SetConfig(AppConfig, String),
    CheckLibUpdate(String),
    CheckAppUpdate(String),
}
