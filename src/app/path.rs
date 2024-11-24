use std::path::PathBuf;

use directories::ProjectDirs;

pub fn get_app_path() -> PathBuf {
    match ProjectDirs::from("com", "genshin-paisitioning", "") {
        Some(proj_dirs) => proj_dirs.project_path().to_path_buf(),
        None => PathBuf::new(),
    }
}

pub fn get_cache_path() -> PathBuf {
    match ProjectDirs::from("com", "genshin-paisitioning", "") {
        Some(proj_dirs) => proj_dirs.cache_dir().to_path_buf(),
        None => PathBuf::new(),
    }
}

pub fn get_logs_path() -> PathBuf {
    get_app_path().join("logs")
}

pub fn get_current_exe_path() -> String {
    match std::env::current_exe() {
        Ok(current_exe) => {
            match current_exe.file_name() {
                Some(exe_name) => exe_name.to_str().unwrap().to_string(),
                None => String::new(),
            }
        },
        Err(_) => String::new(),
    }
}

pub fn get_current_work_directory() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn get_lib_path() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("cvAutoTrack")
}