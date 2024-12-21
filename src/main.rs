// release test
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unused_crate_dependencies)]

mod app;
mod cvat;
mod models;
mod websocket;
mod views;
mod events;

use app::is_process_already_running;
use crate::views::confirm::confirm_dialog;
use log::*;
use std::{sync::Arc, thread};
use events::EventBus;
use websocket::WebSocketHandler;

#[tokio::main]
async fn main() {
    if is_process_already_running() {
        let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA가 이미 실행중입니다.\n추가로 실행된 프로그램은 잠시 후 종료됩니다."), true);
        std::thread::sleep(std::time::Duration::from_millis(5000));
        return;
    }

    if cfg!(debug_assertions) {
        match app::enable_debug() {
            Ok(_) => {
                debug!("Debug mode enable.");
                debug!("App Version: {}", env!("CARGO_PKG_VERSION"));
            },
            Err(e) => panic!("Debug mode enable failed. {}", e),
        }
        log::debug!("Logging debug messages.");

        initialize(["launch", "debug"].to_vec());
    } else {
        // 디버그 모드가 아닐 때
        // 프로젝트 디렉토리에서 실행된 것이 아닐 경우,
        // 인스톨 과정을 거친다.
        match app::check_proj_directory() {
            Ok(true) => {}
            Ok(false) => {
                if std::env::args().find(|x| x.eq("--update")).is_none() {
                    app::installer::install().unwrap();
                }
                return;
            }
            Err(e) => {
                log::error!("Error: {}", e);
                let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("GPA 설치에 실패했습니다.\n{}", e), true);
                return;
            }
        }

        for a in std::env::args() {
            log::debug!("Argument: {}", a);
            if a.starts_with("genshin-paisitioning://") {
                log::debug!("the program launched with scheme: genshin-paisitioning://");
                let parameters = &a[a.find("://").unwrap() + 3..];
                let param_vec: Vec<&str> = parameters.split('/').collect();
                initialize(param_vec);
            } else {
                if a.eq("--debug") || a.eq("-d") {
                    match app::enable_debug() {
                        Ok(_) => {
                            debug!("Debug mode enable.");
                            debug!("App Version: {}", env!("CARGO_PKG_VERSION"));
                        },
                        Err(e) => {
                            let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("디버그 모드 설정에 실패했습니다.\n{}", e.to_string()), true);
                            panic!("Debug mode enable failed. {}", e)
                        },
                    }
                    log::debug!("Logging debug messages.");
                }
                if a.eq("--install") || a.eq("-i") {
                    log::debug!("Install parameter found.");
                    match app::installer::install() {
                        Ok(_) => {},
                        Err(e) => {
                            let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), &format!("인스톨 파라메터 수행 중 실패했습니다.\n{}", e.to_string()), true);
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
}

fn initialize(param: Vec<&str>) {
    log::debug!("Ready function called with parameters: {:?}", param);
    
    if param.contains(&"debug") {
        log::debug!("Debug mode enable.");
        match app::enable_debug() {
            Ok(_) => {
                debug!("Debug mode enable.");
                debug!("App Version: {}", env!("CARGO_PKG_VERSION"));
            },
            Err(e) => panic!("Debug mode enable failed. {}", e),
        }
    }
    
    if param.contains(&"launch") {
        log::debug!("Launch mode enable.");
        // Ws 시작
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let ws_handler = Arc::new(WebSocketHandler::new());
                let event_bus = Arc::new(EventBus::new());
                
                // 이벤트 핸들러 등록을 먼저 완료
                cvat::register_events(&event_bus, &ws_handler).await
                    .expect("Failed to register CVAT events");
                app::updater::register_events(&event_bus, &ws_handler).await
                    .expect("Failed to register Updater events");
                app::config::register_events(&event_bus, &ws_handler).await
                    .expect("Failed to register Config events");
                
                // 모든 이벤트가 등록된 후 WebSocket 서비스 시작
                websocket::serve(Arc::clone(&ws_handler)).await
                    .expect("Failed to serve WebSocket");
            });
        });

        // 트레이 아이콘 추가
        app::add_tray_item();
    }
}
