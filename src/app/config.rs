use config::{Config, FileFormat};
use std::error::Error;
use std::path::PathBuf;
use crate::app::path;

use crate::events::{EventBus};
use crate::models::{AppConfig, RequestDataTypes, RequestEvent, SendEvent, WsEvent};
use crate::websocket::WebSocketHandler;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use once_cell::sync::OnceCell;

// 전역 Config 상태 관리
static CONFIG: OnceCell<ConfigManager> = OnceCell::new();

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    handlers: Arc<RwLock<Vec<Box<dyn Fn(&AppConfig, &AppConfig) + Send + Sync>>>>,
}

impl ConfigManager {
    pub fn global() -> &'static ConfigManager {
        CONFIG.get_or_init(|| {
            let config = init_config()
                .and_then(|c| Ok(c.try_deserialize::<AppConfig>().unwrap()))
                .unwrap_or_default();
            ConfigManager {
                config: Arc::new(RwLock::new(config)),
                handlers: Arc::new(RwLock::new(Vec::new())),
            }
        })
    }

    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    pub async fn register_handler(&self, handler: impl Fn(&AppConfig, &AppConfig) + Send + Sync + 'static) {
        let mut handlers = self.handlers.write().await;
        handlers.push(Box::new(handler));
    }

    pub async fn update(&self, mut f: impl FnMut(&mut AppConfig)) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut config = self.config.write().await;
        let old_config = config.clone();
        f(&mut config);
        
        // 변경된 값들 로깅
        for (field, (old, new)) in [
            ("auto_app_update", (old_config.auto_app_update, config.auto_app_update)),
            ("auto_lib_update", (old_config.auto_lib_update, config.auto_lib_update)),
            ("use_bit_blt_capture_mode", (old_config.use_bit_blt_capture_mode, config.use_bit_blt_capture_mode)),
        ] {
            if old != new {
                log::debug!("{}: {} -> {}", field, old, new);
            }
        }

        // 핸들러들 실행
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler(&old_config, &config);
        }

        save_config(&config)?;
        Ok(())
    }
}

// 설정 파일 초기화
pub fn init_config() -> Result<Config, std::io::Error> {
    log::debug!("Config File: 초기화 시작");
    let target_dir = path::get_app_path();

    match std::fs::create_dir_all(path::get_cache_path()) {
        Ok(_) => {},
        Err(e) => {
            log::error!("Project Directory: 생성 실패");
            log::error!("Error: {}", e);
        }
    }
    match create_config_file_if_not_exist(&target_dir.join("config.json"))
    {
        Ok(_) => {},
        Err(e) => {
            log::error!("Config File: 생성 실패");
            log::error!("Error: {}", e);
        }
    }

    let settings = Config::builder()
        .add_source(config::File::new(target_dir.join("config.json").to_str().unwrap(), FileFormat::Json))
        /* .add_source(config::File::with_name(
            target_dir.join("config.json").to_str().unwrap(),
        )) */
        .add_source(config::Environment::with_prefix("gpa").separator("_"))
        .build();
    
    match settings {
        Ok(settings) => Ok(settings),
        Err(e) => {
            log::error!("Config File: 로드 실패");
            log::error!("Error: {}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}

// 설정 파일 생성
pub fn create_config_file_if_not_exist(target_path: &PathBuf) -> Result<(), std::io::Error>{
    if !target_path.exists() {
        log::debug!("Config File: 생성");
        let app_config = AppConfig {
            auto_app_update: true,
            auto_lib_update: true,
            capture_interval: 250,
            capture_delay_on_error: 1000,
            use_bit_blt_capture_mode: false
        };

        let contents = serde_json::to_string_pretty(&app_config);
        match contents {
            Ok(contents) => {
                return std::fs::write(target_path, contents);
            },
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    } else {
        return Ok(());
    }
}

// 이벤트 등록
pub async fn register_events(event_bus: &Arc<EventBus>, ws_handler: &Arc<WebSocketHandler>) -> Result<(), Box<dyn Error + Send + Sync>> { 
    let ws_handler_get = ws_handler.clone();
    ws_handler.register("getConfig", move |id, _| {
        let ws_handler = ws_handler_get.clone();
        async move {
            let config = ConfigManager::global().get().await;
            ws_handler.send_to(id.clone(), SendEvent::from(WsEvent::Config { 
                config, 
                id: id.clone() 
            })).await?;
            Ok(())
        }
    }).await?;

    let ws_handler_set = Arc::new(ws_handler.clone());
    ws_handler.register("setConfig", move |id, params: RequestEvent| {
        let ws_handler = ws_handler_set.clone();
        async move {
            log::debug!("Set Config Event");
            let config = match &params.data {
                Some(RequestDataTypes::AppConfig(data)) => data.clone(),
                Some(_) => return Err("Invalid config data type".into()),
                None => return Err("Config data is required".into())
            };
            ConfigManager::global().update(|c| {
                *c = config.clone();
            }).await?;
            ws_handler.send_to(id.clone(), SendEvent::from(WsEvent::Config { 
                config, 
                id: id.clone() 
            })).await?;
            Ok(())
        }
    }).await?;

    Ok(())
}

// 설정 파일 저장
pub fn save_config(app_config: &AppConfig) -> Result<(), Box<dyn Error + Send + Sync>> {
    let target_dir = path::get_app_path();
    let result = std::fs::write(
        target_dir.join("config.json"),
        serde_json::to_string_pretty(&app_config).unwrap(),
    );
    match result {
        Ok(_) => {
            log::debug!("Config File: 저장 완료");
            Ok(())
        },
        Err(e) => {
            log::error!("Config File: 저장 실패");
            log::error!("Error: {}", e);
            Err(Box::new(e))
        }
    }
}
