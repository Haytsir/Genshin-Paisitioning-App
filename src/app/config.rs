use config::Config;
use std::error::Error;
use std::path::PathBuf;
use crate::app::path;

use crate::models::AppConfig;

pub fn init_config() -> Result<Config, std::io::Error> {
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
        .add_source(config::File::with_name(
            target_dir.join("config.json").to_str().unwrap(),
        ))
        .add_source(config::Environment::with_prefix("APP"))
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

pub fn create_config_file_if_not_exist(target_path: &PathBuf) -> Result<(), std::io::Error>{
    if !target_path.exists() {
        let app_config = AppConfig {
            auto_app_update: true,
            auto_lib_update: true,
            capture_interval: 250,
            capture_delay_on_error: 1000,
            use_bit_blt_capture_mode: false,
            changed: None,
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

pub fn save_config(app_config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let target_dir = path::get_app_path();
    std::fs::write(
        target_dir.join("config.json"),
        serde_json::to_string_pretty(&app_config).unwrap(),
    )
    .unwrap();
    Ok(())
}
