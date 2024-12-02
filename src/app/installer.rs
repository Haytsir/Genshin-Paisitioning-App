use crate::app::path;
use std::path::Path;
use std::time::Duration;
use winreg::enums::*;
use winreg::RegKey;

use crate::app::config::create_config_file_if_not_exist;
use crate::views::confirm::confirm_dialog;

use super::check_elevation;

pub fn install() -> Result<(), std::io::Error>{
    let target_dir = path::get_app_path();
    let current_exe = std::env::current_exe().unwrap();
    let exe_name = current_exe.file_name().unwrap();
    if check_elevation(&target_dir.join(exe_name), vec!["--install".to_string()]) {
        log::debug!("Installing...");
        let exe_path = &target_dir.join(exe_name);
        let result = std::fs::copy(&current_exe, &target_dir.join(exe_name));
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\n실행 파일 복사에 실패했습니다.\n{}", e.to_string()), true);
                return Err(e.into());
            }
        }
        let result = register_url_scheme(exe_path);
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\nURL 스키마 생성 실패\n{}", e.to_string()), true);
                return Err(e.into());
            }
        }
        let result = register_uninstall_item(exe_path);
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\nuninstall item 생성 실패\n{}", e.to_string()), true);
                return Err(e.into());
            }
        }
        let result = create_config_file_if_not_exist(&target_dir.join("config.json"));
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\n설정 파일 생성 실패\n{}", e.to_string()), true);
                return Err(e.into());
            }
        }
        let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 설치를 완료했습니다.", false);
        let _ = self_replace::self_delete();
        std::thread::sleep(Duration::from_millis(5000));
    } else {
        let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 설치를 취소했습니다.\n관리자 권한이 필요합니다.", true);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "관리자 권한이 필요합니다."));
    }
    Ok(())
}

pub fn uninstall() -> Result<(), std::io::Error> {
    let target_dir = path::get_app_path();
    let current_exe = std::env::current_exe().unwrap();
    let exe_name = current_exe.file_name().unwrap();
    if check_elevation(&target_dir.join(exe_name), vec!["--uninstall".to_string()]) {
        log::debug!("Uninstalling...");
        let result = remove_url_scheme();
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 제거에 실패했습니다.", true);
                return Err(e.into());
            }
        }
        let result = remove_uninstall_item();
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 제거에 실패했습니다.", true);
                return Err(e.into());
            }
        }
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 제거에 실패했습니다.", true);
                return Err(e.into());
            }
        }
        let _ = self_replace::self_delete_at(target_dir);
    } else {
        let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 제거를 취소했습니다.\n관리자 권한이 필요합니다.", true);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "관리자 권한이 필요합니다."));
    }
    Ok(())
}

fn register_url_scheme(install_path: &Path) -> Result<(), std::io::Error> {
    log::debug!("Registering URL Scheme...");
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
            return result;
        }
    }

    result = key.set_value("URL Protocol", &"");
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
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
            return result;
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
            return result;
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
            return result;
        }
    }

    Ok(())
}

fn register_uninstall_item(install_path: &Path) -> Result<(), std::io::Error> {
    log::debug!("Registering uninstall item...");
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
            return result;
        }
    }

    result = key.set_value(
        "UninstallString",
        &format!("\"{}\" --uninstall", &install_path.to_str().unwrap()),
    );
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value("NoModify", &"1");
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value("NoRepair", &"1");
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"));
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value("DisplayIcon", &install_path.to_str().unwrap());
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value("Publisher", &env!("CARGO_PKG_AUTHORS"));
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }

    result = key.set_value(
        "InstallLocation",
        &install_path.parent().unwrap().to_str().unwrap(),
    );
    match result {
        Ok(_) => {}
        Err(_) => {
            return result;
        }
    }
    Ok(())
}

fn remove_url_scheme() -> Result<(), std::io::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software\\Classes\\genshin-paisitioning");
    hkcu.delete_subkey_all(path)
}

fn remove_uninstall_item() -> Result<(), std::io::Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path =
        Path::new("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\genshin-paisitioning");
    hklm.delete_subkey_all(path)
}
