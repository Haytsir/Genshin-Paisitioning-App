// release test
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unused_crate_dependencies)]

mod app;
mod cvat;
mod models;
mod websocket;
mod views;

use app::{is_process_already_running, updater::updater_event_handler, path};
use crossbeam_channel::{unbounded, Receiver, Sender};
use cvat::cvat_event_handler;
use crate::views::confirm::confirm_dialog;
use models::{AppEvent, WsEvent};
use threadpool::ThreadPool;
use log::*;

fn main() {
    #[cfg(debug_assertions)]
    {
        match app::enable_debug() {
            Ok(_) => log::debug!("Debug mode enable."),
            Err(e) => panic!("Debug mode enable failed. {}", e),
        }
        log::debug!("Logging debug messages.");
    }
    // 프로젝트 디렉토리에서 실행된 것이 아닐 경우,
    // 인스톨 과정을 거친다.
    #[cfg(not(debug_assertions))]
    match app::check_proj_directory() {
        Ok(true) => {}
        Ok(false) => {
            let target_dir = path::get_app_path();
            let current_exe = std::env::current_exe().unwrap();
            let exe_name = current_exe.file_name().unwrap();
            if std::env::args().find(|x| x.eq("--update")).is_none() {
                install(false);
            } else {
                install(true);
            }
            return;
        }
        Err(e) => {
            log::error!("Error: {}", e);
            let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\n{}", e), true);
            return;
        }
    }

    if is_process_already_running() {
        return;
    }

    // 인자를 파싱한다.
    #[cfg(debug_assertions)]
    ready(["launch"].to_vec());
    #[cfg(not(debug_assertions))]
    for a in std::env::args() {
        log::debug!("Argument: {}", a);
        if a.starts_with("genshin-paisitioning://") {
            log::debug!("the program launched with scheme: genshin-paisitioning://");
            let parameters = &a[a.find("://").unwrap() + 3..];
            let param_vec: Vec<&str> = parameters.split('/').collect();
            ready(param_vec);
        } else {
            if a.eq("--debug") || a.eq("-d") {
                match app::enable_debug() {
                    Ok(_) => log::debug!("Debug mode enable."),
                    Err(e) => panic!("Debug mode enable failed. {}", e),
                }
                log::debug!("Logging debug messages.");
            }
            if a.eq("--install") || a.eq("-i") {
                log::debug!("Install parameter found.");
                match app::installer::install() {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
                return;
            } else if a.eq("--uninstall") || a.eq("-u") {
                log::debug!("Uninstall parameter found.");
                match app::installer::uninstall() {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
                return;
            }
        }
    }
}

fn ready(param: Vec<&str>) {
    log::debug!("Argument: {:?}", param);
    let (cvat_sender, cvat_receiver): (Sender<AppEvent>, Receiver<AppEvent>) = unbounded();
    let (ws_sender, ws_receiver): (Sender<WsEvent>, Receiver<WsEvent>) = unbounded();
    let pool: ThreadPool = ThreadPool::new(5);

    let config_result = app::config::init_config();
    let config: config::Config;
    match config_result {
        Ok(conf) => {
            config = conf;
        }
        Err(e) => {
            log::error!("Config: 로드 실패");
            log::error!("Error: {}", e);
            return;
        }
    }

    if param.contains(&"debug") {
        match app::enable_debug() {
            Ok(_) => log::debug!("Debug mode enable."),
            Err(e) => panic!("Debug mode enable failed. {}", e),
        }
    }
    if param.contains(&"launch") {
        log::debug!("Launch parameter found.");
        let ws_handler_sender = cvat_sender;
        let ws_handler_receiver = ws_receiver;

        // Ws 시작
        pool.execute(move || {
            log::debug!("start ws server");
            websocket::serve(ws_handler_sender, ws_handler_receiver);
        });

        // Ws과 Cvat Library의 연동 핸들러 시작
        pool.execute(move || {
            log::debug!("start ws handler");
            let updater_handler_sender = ws_sender.clone();
            let updater_handler_receiver = cvat_receiver.clone();
            match updater_event_handler(config.clone(), Some(updater_handler_sender), Some(updater_handler_receiver)) {
                Ok(_) => {
                    let cvat_handler_sender = ws_sender.clone();
                    let cvat_handler_receiver = cvat_receiver.clone();
                    cvat_event_handler(
                        config.clone(),
                        Some(cvat_handler_sender),
                        Some(cvat_handler_receiver),
                    );
                },
                Err(e) => {
                    log::error!("Error: {}", e);
                    panic!("Updater event handler failed. {}", e);
                }                
            }
        });

        // 트레이 아이콘 추가
        app::add_tray_item();
    }

    pool.join();
}
