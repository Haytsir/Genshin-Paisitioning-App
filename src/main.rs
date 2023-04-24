// release test
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unused_crate_dependencies)]

mod app;
mod cvat;
mod models;
mod websocket;
use app::{is_process_already_running, updater::updater_event_handler};
use crossbeam_channel::{unbounded, Receiver, Sender};
use cvat::cvat_event_handler;
use directories::ProjectDirs;
use models::{AppEvent, WsEvent};
use threadpool::ThreadPool;

use log::LevelFilter;

fn main() {
    // 프로젝트 디렉토리에서 실행된 것이 아닐 경우,
    // 인스톨 과정을 거친다.
    if !app::check_proj_directory() {
        let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
        let target_dir = proj_dirs.cache_dir().parent().unwrap();
        let current_exe = std::env::current_exe().unwrap();
        let exe_name = current_exe.file_name().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
        if std::env::args().find(|x| x.eq("--update")).is_none() {
            if !app::check_elevation(&target_dir.join(exe_name), vec!["--install"]) {
                return;
            }
        }
    }

    if is_process_already_running() {
        return;
    }

    // 인자를 파싱한다.
    for a in std::env::args() {
        if a.starts_with("genshin-paisitioning://") {
            let parameters = &a[a.find("://").unwrap() + 3..];
            let param_vec: Vec<&str> = parameters.split('/').collect();
            ready(param_vec);
        } else {
            if a.eq("--debug") || a.eq("-d") {
                env_logger::builder()
                    .filter_level(LevelFilter::Debug)
                    .init();
                log::debug!("Logging debug messages.");
            }
            if a.eq("--install") || a.eq("-i") {
                log::debug!("Install parameter found.");
                app::installer::install();
                return;
            } else if a.eq("--uninstall") || a.eq("-u") {
                log::debug!("Uninstall parameter found.");
                app::installer::uninstall();
                return;
            }
        }
    }

    cvat::get_compile_version();
    cvat::get_compile_time();
}

fn ready(param: Vec<&str>) {
    let (cvat_sender, cvat_receiver): (Sender<AppEvent>, Receiver<AppEvent>) = unbounded();
    let (ws_sender, ws_receiver): (Sender<WsEvent>, Receiver<WsEvent>) = unbounded();
    let pool: ThreadPool = ThreadPool::new(5);

    let config = app::config::init_config();

    if param.contains(&"debug") {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    }
    if param.contains(&"launch") {
        let ws_handler_sender = cvat_sender;
        let ws_handler_receiver = ws_receiver;

        // Ws 시작
        pool.execute(move || {
            websocket::serve(ws_handler_sender, ws_handler_receiver);
        });

        // Ws과 Cvat Library의 연동 핸들러 시작
        pool.execute(move || {
            let updater_handler_sender = ws_sender.clone();
            let updater_handler_receiver = cvat_receiver.clone();
            let ready =
                updater_event_handler(config.clone(), Some(updater_handler_sender), Some(updater_handler_receiver));
            if ready {
                let cvat_handler_sender = ws_sender.clone();
                let cvat_handler_receiver = cvat_receiver.clone();
                cvat_event_handler(
                    config,
                    Some(cvat_handler_sender),
                    Some(cvat_handler_receiver),
                );
            }
        });

        // 트레이 아이콘 추가
        app::add_tray_item();
    }

    pool.join();
}
