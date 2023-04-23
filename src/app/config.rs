use config::Config;
use directories::ProjectDirs;
use std::error::Error;
use std::path::PathBuf;

use crate::models::AppConfig;

pub fn init_config() -> Config {
    let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
    let target_dir = proj_dirs.cache_dir().parent().unwrap();

    create_config_file_if_not_exist(&target_dir.join("config.json"));

    let settings = Config::builder()
        .add_source(config::File::with_name(
            target_dir.join("config.json").to_str().unwrap(),
        ))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    settings
}

pub fn create_config_file_if_not_exist(target_path: &PathBuf) {
    if !target_path.exists() {
        let app_config = AppConfig {
            capture_interval: 250,
            capture_delay_on_error: 1000,
            use_bit_blt_capture_mode: false,
        };
        std::fs::write(
            target_path,
            serde_json::to_string_pretty(&app_config).unwrap(),
        )
        .unwrap();
    }
}

pub fn save_config(app_config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
    let target_dir = proj_dirs.cache_dir().parent().unwrap();
    std::fs::write(
        target_dir.join("config.json"),
        serde_json::to_string_pretty(&app_config).unwrap(),
    )
    .unwrap();
    Ok(())
}
