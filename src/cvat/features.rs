use config::Config;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

use super::implements::LibCvat;

use crate::models::{AppConfig, AppEvent, TrackData, WsEvent};
use libc::{c_char, c_double, c_int};
use std::ffi::CStr;
use std::option::Option;

use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

extern crate lazy_static;
use lazy_static::lazy_static;

lazy_static! {
    #[derive(Copy, Clone, Debug)]
    pub static ref LIB:libloading::Library = unsafe{ libloading::Library::new("./cvAutoTrack/cvAutoTrack.dll") }.expect ( "ERROR loading cvAutoTrack.dll" );
}

static TRACKDATA: OnceCell<Mutex<TrackData>> = OnceCell::new();
static THREAD_POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();
static IS_TRACKING: OnceCell<Mutex<bool>> = OnceCell::new();
static LIBCVAT: OnceCell<Mutex<LibCvat>> = OnceCell::new();
static CAPTURE_INTERVAL: OnceCell<Mutex<u64>> = OnceCell::new();
static CAPTURE_DELAY_ON_ERROR: OnceCell<Mutex<u64>> = OnceCell::new();

pub fn start_track_thead(sender: Option<Sender<WsEvent>>, use_bit_blt: bool) -> bool {
    let cvat = get_lib();
    log::debug!("start_track_thead: start");
    if get_is_tracking() {
        log::debug!("start_track_thead again?");
        return true;
    }
    if cvat.init() {
        log::debug!("start_track_thead init done");
        set_is_tracking(true);
        if use_bit_blt {
            // cvat.set_use_bitblt_capture_mode();
        } else {
            // cvat.set_use_dx11_capture_mode();
        }
        cvat.set_disable_file_log();
        (*ensure_thread_pool())
            .lock()
            .unwrap()
            .execute(move || loop {
                if !get_is_tracking() {
                    break;
                }
                if track_process(sender.clone()) {
                    thread::sleep(Duration::from_millis(get_capture_interval()));
                } else {
                    thread::sleep(Duration::from_millis(get_capture_delay_on_error()));
                }
            });
    }
    true
}

pub fn stop_track_thread(/*sender: Option<Sender<WsEvent>>*/) -> bool {
    if !get_is_tracking() {
        return true;
    }
    let cvat = get_lib();
    if cvat.uninit() {
        drop((*ensure_thread_pool()).lock().unwrap());
        set_is_tracking(false);
        return true;
    }
    false
}

// 클라이언트로부터 이벤트를 전송받았을 경우
pub fn cvat_event_handler(
    mut config: Config,
    tx: Option<Sender<WsEvent>>,
    rx: Option<Receiver<AppEvent>>,
) {
    while let Some(r) = rx.as_ref() {
        log::info!("LIB LOOP!");
        let res = r.recv();
        match res {
            Ok(AppEvent::Init()) => {
                let app_config: AppConfig = config.clone().try_deserialize().unwrap();
                set_capture_interval(u64::from(app_config.capture_interval));
                set_capture_delay_on_error(u64::from(app_config.capture_delay_on_error));
                log::debug!("Got Init");
                start_track_thead(tx.clone(), app_config.use_bit_blt_capture_mode);
            }
            Ok(AppEvent::Uninit()) => {
                log::debug!("Got Uninit");
                stop_track_thread(/*tx.clone()*/);
            }
            Ok(AppEvent::GetConfig(id)) => {
                log::debug!("Got GetConfig");
                if let Some(t) = tx.as_ref() {
                    let app_config: AppConfig = config.clone().try_deserialize().unwrap();
                    t.send(WsEvent::Config(app_config, id)).unwrap();
                }
            }
            Ok(AppEvent::SetConfig(mut new_app_config, id)) => {
                log::debug!("Got SetConfig");
                if let Some(t) = tx.as_ref() {
                    let new_config = Config::builder()
                        .add_source(config.clone())
                        .set_override("captureInterval", new_app_config.capture_interval)
                        .expect("Failed to set overide")
                        .set_override("captureDelayOnError", new_app_config.capture_delay_on_error)
                        .expect("Failed to set overide")
                        .set_override(
                            "useBitBltCaptureMode",
                            new_app_config.use_bit_blt_capture_mode,
                        )
                        .expect("Failed to set overide")
                        .build()
                        .unwrap();
                    on_config_changed(config.clone(), new_config.clone());
                    config = new_config;
                    new_app_config.changed = Some(true);
                    t.send(WsEvent::Config(new_app_config, id)).unwrap();
                }
            }
            Ok(_) => {
                log::error!("Unknown: {:#?}", res);
            }
            Err(e) => {
                log::error!("Unknown: {}", e);
            } //panic!("panic happened"),
        }
    }
}

fn on_config_changed(config: Config, new_config: Config) {
    let _cvat = get_lib();
    let old_app_config: AppConfig = config.try_deserialize().unwrap();
    let mut new_app_config: AppConfig = new_config.try_deserialize().unwrap();
    if new_app_config.capture_interval < 100 {
        new_app_config.capture_interval = 100;
    }
    if new_app_config.capture_delay_on_error < 100 {
        new_app_config.capture_delay_on_error = 100;
    }
    set_capture_interval(u64::from(new_app_config.capture_interval));
    set_capture_delay_on_error(u64::from(new_app_config.capture_delay_on_error));
    if old_app_config.use_bit_blt_capture_mode != new_app_config.use_bit_blt_capture_mode {
        if new_app_config.use_bit_blt_capture_mode {
            // cvat.set_use_bitblt_capture_mode();
        } else {
            // cvat.set_use_dx11_capture_mode();
        }
    }
    let _ = crate::app::config::save_config(&new_app_config);
}

fn ensure_lib_load() -> &'static Mutex<LibCvat> {
    LIBCVAT.get_or_init(|| Mutex::new(LibCvat::new(LIB)))
}

fn get_lib() -> LibCvat {
    *ensure_lib_load().lock().unwrap()
}

fn ensure_is_tracking() -> &'static Mutex<bool> {
    IS_TRACKING.get_or_init(|| Mutex::new(false))
}

fn ensure_thread_pool() -> &'static Mutex<ThreadPool> {
    THREAD_POOL.get_or_init(|| Mutex::new(ThreadPool::new(1)))
}

fn ensure_track_data() -> &'static Mutex<TrackData> {
    TRACKDATA.get_or_init(|| {
        Mutex::new(TrackData {
            x: 0.0,
            y: 0.0,
            a: 0.0,
            r: 0.0,
            m: 0,
            err: "",
        })
    })
}

pub fn get_is_tracking() -> bool {
    *ensure_is_tracking().lock().unwrap()
}

pub fn set_is_tracking(val: bool) {
    *ensure_is_tracking().lock().unwrap() = val;
}

fn ensure_capture_interval() -> &'static Mutex<u64> {
    CAPTURE_INTERVAL.get_or_init(|| Mutex::new(250))
}

pub fn get_capture_interval() -> u64 {
    *ensure_capture_interval().lock().unwrap()
}

pub fn set_capture_interval(val: u64) {
    if val > 100 {
        *ensure_capture_interval().lock().unwrap() = val;
    } else {
        *ensure_capture_interval().lock().unwrap() = 100;
    }
    
}

fn ensure_capture_delay_on_error() -> &'static Mutex<u64> {
    CAPTURE_DELAY_ON_ERROR.get_or_init(|| Mutex::new(800))
}

pub fn get_capture_delay_on_error() -> u64 {
    *ensure_capture_delay_on_error().lock().unwrap()
}

pub fn set_capture_delay_on_error(val: u64) {
    if val > 100 {
        *ensure_capture_delay_on_error().lock().unwrap() = val;
    } else {
        *ensure_capture_delay_on_error().lock().unwrap() = 100;
    }
}

pub fn track_process(sender: Option<Sender<WsEvent>>) -> bool {
    let mut trackdata = *ensure_track_data().lock().unwrap();
    let cvat = get_lib();
    if !track(
        &mut trackdata.x,
        &mut trackdata.y,
        &mut trackdata.a,
        &mut trackdata.r,
        &mut trackdata.m,
    ) {
        let mut cs = [0; 256];
        let c_buf: *const c_char = cs.as_mut_ptr();
        cvat.get_last_err_json(c_buf, 256);
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        trackdata.err = c_str.to_str().unwrap(); // .to_string();

        let _ = sender.unwrap().send(WsEvent::Track(trackdata));
        return false;
    }
    let _ = sender.unwrap().send(WsEvent::Track(trackdata));
    true
}

pub fn track(
    x: &mut c_double,
    y: &mut c_double,
    a: &mut c_double,
    r: &mut c_double,
    m: &mut c_int,
) -> bool {
    let mut result: bool = true;
    let cvat = get_lib();
    if !cvat.get_transform_of_map(x, y, a, m) {
        result = false;
    }
    if !cvat.get_rotation(r) {
        result = false;
    }
    result
}

pub fn get_compile_version() {
    let mut cs = [0; 256];
    let c_buf: *const c_char = cs.as_mut_ptr();
    let cvat = get_lib();
    cvat.get_compile_version(c_buf, 256);
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = c_str.to_str().unwrap(); // .to_owned() if want to own the str.
    log::debug!("Compile Version: {}", str_slice);
}

pub fn get_compile_time() {
    let mut cs = [0; 256];
    let c_buf: *const c_char = cs.as_mut_ptr();
    let cvat = get_lib();
    cvat.get_compile_time(c_buf, 256);
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = c_str.to_str().unwrap(); // .to_owned() if want to own the str.
    log::debug!("Compile Time: {}", str_slice);
}
