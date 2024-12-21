#[allow(dead_code)]
pub use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use crate::cvat::bindings::cvAutoTrack;
use parking_lot::RwLock;
use crate::app::config::ConfigManager;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: &'static str,
    pub lib_version: &'static str,
    pub configs: AppConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct AppConfig {
    pub auto_app_update: bool,
    pub auto_lib_update: bool,
    pub capture_interval: u32,
    pub capture_delay_on_error: u32,
    pub use_bit_blt_capture_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_app_update: true,
            auto_lib_update: true,
            capture_interval: 250,
            capture_delay_on_error: 1000,
            use_bit_blt_capture_mode: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub target_type: String,
    pub target_version: String,
    pub display_version_name: String,
    pub current_version: String,
    pub downloaded: u64,
    pub file_size: u64,
    pub percent: f64,
    pub done: bool,
    pub updated: bool
}

// 내부 앱 이벤트 (컨텍스트 간 통신)
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum AppEvent {
    Init(),
    Uninit(),
    GetConfig(String),
    SetConfig(AppConfig, String),
    CheckLibUpdate(String, bool),
    CheckAppUpdate(String, bool),
}

pub struct AppState {
    pub capture_interval: Arc<AtomicU32>,
    pub capture_delay_on_error: Arc<AtomicU32>,
    pub is_tracking: Arc<AtomicBool>,
    instance: RwLock<Option<cvAutoTrack>>,
}

impl AppState {
    pub fn new() -> Self {
        let capture_interval = Arc::new(AtomicU32::new(250));
        let capture_delay_on_error = Arc::new(AtomicU32::new(1000));
        let is_tracking = Arc::new(AtomicBool::new(false));

        let state = Self {
            capture_interval: Arc::clone(&capture_interval),
            capture_delay_on_error: Arc::clone(&capture_delay_on_error),
            is_tracking,
            instance: RwLock::new(None),
        };

        // 설정 변경 핸들러는 별도로 등록
        tokio::spawn(async move {
            ConfigManager::global().register_handler(move |_, new_config| {
                capture_interval.store(new_config.capture_interval, Ordering::Relaxed);
                capture_delay_on_error.store(new_config.capture_delay_on_error, Ordering::Relaxed);
            }).await;
        });

        state
    }

    // 캡처 관련 메서드
    pub fn get_capture_interval(&self) -> u32 {
        self.capture_interval.load(Ordering::Relaxed)
    }

    pub fn get_capture_delay_on_error(&self) -> u32 {
        self.capture_delay_on_error.load(Ordering::Relaxed)
    }

    // CVAT 관련 메서드
    pub fn set_tracking(&self, value: bool) {
        self.is_tracking.store(value, Ordering::Relaxed);
    }

    pub fn is_tracking(&self) -> bool {
        self.is_tracking.load(Ordering::Relaxed)
    }

    pub fn get_instance(&self) -> parking_lot::RwLockReadGuard<'_, Option<cvAutoTrack>> {
        self.instance.read()
    }

    pub fn set_instance(&self, instance: Option<cvAutoTrack>) {
        *self.instance.write() = instance;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
