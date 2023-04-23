use directories::ProjectDirs;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

use crate::app::config::create_config_file_if_not_exist;
use crate::app::run_cmd;

use super::check_elevation;

pub fn install() {
    let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
    let target_dir = proj_dirs.cache_dir().parent().unwrap();
    let current_exe = std::env::current_exe().unwrap();
    let exe_name = current_exe.file_name().unwrap();
    if check_elevation(&target_dir.join(exe_name), vec!["--install"]) {
        log::debug!("Installing...");
        let exe_path = &target_dir.join(exe_name);
        register_url_scheme(exe_path);
        register_uninstall_item(exe_path);
        create_config_file_if_not_exist(&target_dir.join("config.json"));
        let _ = msgbox::create(env!("CARGO_PKG_DESCRIPTION"), "GPA 설치를 완료했습니다.", msgbox::IconType::None);
    }
}

pub fn uninstall() {
    let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
    let target_dir = proj_dirs.cache_dir().parent().unwrap();
    let current_exe = std::env::current_exe().unwrap();
    let exe_name = current_exe.file_name().unwrap();
    if check_elevation(&target_dir.join(exe_name), vec!["--uninstall"]) {
        log::debug!("Uninstalling...");
        remove_url_scheme();
        remove_uninstall_item();
        run_cmd(
            format!(
                "ping localhost -n 3 > nul & del /F /Q /S {}",
                target_dir.to_str().unwrap()
            )
            .as_str(),
        );
    }
}

fn register_url_scheme(install_path: &Path) -> bool {
    // register protocol using registry
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut path = Path::new("Software\\Classes\\genshin-paisitioning");
    let (mut key, mut disp) = hkcu.create_subkey(path).unwrap();
    match disp {
        REG_CREATED_NEW_KEY => log::debug!("A class key has been created"),
        REG_OPENED_EXISTING_KEY => log::debug!("An existing class key has been opened"),
    }

    let mut result = key.set_value("", &"URL: genshin-paisitioning Protocol");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("URL Protocol", &"");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    path = Path::new("DefaultIcon");
    let default_icon_key: RegKey;
    (default_icon_key, disp) = key.create_subkey(path).unwrap();
    match disp {
        REG_CREATED_NEW_KEY => log::debug!("A default icon key has been created"),
        REG_OPENED_EXISTING_KEY => log::debug!("An existing default icon key has been opened"),
    }
    result = default_icon_key.set_value("", &format!("\"{}\",0", install_path.to_str().unwrap()));
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    path = Path::new("shell\\open");
    (key, disp) = key.create_subkey(path).unwrap();
    match disp {
        REG_CREATED_NEW_KEY => log::debug!("A shell open key has been created"),
        REG_OPENED_EXISTING_KEY => log::debug!("An existing shell open key has been opened"),
    }
    result = key.set_value("FriendlyAppName", &"원신 파이지셔닝");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    path = Path::new("command");
    (key, disp) = key.create_subkey(path).unwrap();
    match disp {
        REG_CREATED_NEW_KEY => log::debug!("A command key has been created"),
        REG_OPENED_EXISTING_KEY => log::debug!("An existing command key has been opened"),
    }
    result = key.set_value(
        "",
        &format!("\"{}\" \"%1\"", install_path.to_str().unwrap()),
    );
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    true
}

fn register_uninstall_item(install_path: &Path) -> bool {
    // write uninstall info to registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path =
        Path::new("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\genshin-paisitioning");
    let (key, disp) = hklm.create_subkey(path).unwrap();
    match disp {
        REG_CREATED_NEW_KEY => log::debug!("An uninstall key has been created"),
        REG_OPENED_EXISTING_KEY => log::debug!("An existing uninstall key has been opened"),
    }
    let mut result = key.set_value("DisplayName", &"원신 파이지셔닝");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value(
        "UninstallString",
        &format!("\"{}\" --uninstall", &install_path.to_str().unwrap()),
    );
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("NoModify", &"1");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("NoRepair", &"1");
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"));
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("DisplayIcon", &install_path.to_str().unwrap());
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value("Publisher", &env!("CARGO_PKG_AUTHORS"));
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }

    result = key.set_value(
        "InstallLocation",
        &install_path.parent().unwrap().to_str().unwrap(),
    );
    match result {
        Ok(_) => {}
        Err(_) => {
            return false;
        }
    }
    true
}

fn remove_url_scheme() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software\\Classes\\genshin-paisitioning");
    let result = hkcu.delete_subkey_all(path);
    match result {
        Ok(_) => {
            true
        }
        Err(_) => {
            false
        }
    }
}

fn remove_uninstall_item() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path =
        Path::new("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\genshin-paisitioning");
    let result = hklm.delete_subkey_all(path);
    match result {
        Ok(_) => {
            true
        }
        Err(_) => {
            false
        }
    }
}
