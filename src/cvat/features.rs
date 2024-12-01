use config::Config;
use once_cell::sync::OnceCell;
use std::error::Error;
use std::sync::Mutex;

use super::bindings::cvAutoTrack;

use crate::app::set_lib_directory;
use crate::models::{AppConfig, AppEvent, TrackData, WsEvent};
use libc::{c_double, c_int};
use std::ffi::CStr;
use std::option::Option;

use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use libloading::Library;

static THREAD_POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();
static IS_TRACKING: OnceCell<Mutex<bool>> = OnceCell::new();
static CAPTURE_INTERVAL: OnceCell<Mutex<u64>> = OnceCell::new();
static CAPTURE_DELAY_ON_ERROR: OnceCell<Mutex<u64>> = OnceCell::new();
static CVAT_INSTANCE: OnceCell<Mutex<Option<(cvAutoTrack, Library)>>> = OnceCell::new();

impl Clone for cvAutoTrack {
    fn clone(&self) -> Self {
        unsafe { cvAutoTrack::new("cvAutoTrack.dll").unwrap() }
    }
}

pub fn start_track_thead(sender: Option<Sender<WsEvent>>, use_bit_blt: bool) -> bool {
    log::debug!("start_track_thead: start");
    if get_is_tracking() {
        log::debug!("start_track_thead again?");
        return true;
    }

    let cvat = {
        let guard = ensure_cvat_instance().lock().unwrap();
        if let Some((cvat, _)) = guard.as_ref() {
            cvat.clone()
        } else {
            initialize_cvat().unwrap();
            let guard = ensure_cvat_instance().lock().unwrap();
            if let Some((cvat, _)) = guard.as_ref() {
                cvat.clone()
            } else {
                return false;
            }
        }
    };

    if unsafe { cvat.init() } {
        log::debug!("start_track_thead init done");
        set_is_tracking(true);
        if use_bit_blt {
            // cvat.set_use_bitblt_capture_mode();
        } else {
            // cvat.set_use_dx11_capture_mode();
        }
        
        (*ensure_thread_pool())
            .lock()
            .unwrap()
            .execute(move || loop {
                if !get_is_tracking() {
                    unsafe { cvat.uninit() };
                    break;
                }
                match track_process(&cvat, sender.clone()) {
                    Ok(_) => {
                        thread::sleep(Duration::from_millis(get_capture_interval()));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(get_capture_delay_on_error()));
                    }
                }
            });
        return true;
    }
    false
}

// TODO: sender를 통해 &cvAutoTrack 를 전송할 수 있는가?
pub fn stop_track_thread(cvat: &cvAutoTrack /*sender: Option<Sender<WsEvent>>*/) -> bool {
    if !get_is_tracking() {
        return true;
    }
    if unsafe { cvat.uninit() } {
        drop((*ensure_thread_pool()).lock().unwrap());
        set_is_tracking(false);
        return true;
    }
    false
}

pub fn on_config_changed(config: Config, new_config: Config) {
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

fn ensure_is_tracking() -> &'static Mutex<bool> {
    IS_TRACKING.get_or_init(|| Mutex::new(false))
}

fn ensure_thread_pool() -> &'static Mutex<ThreadPool> {
    THREAD_POOL.get_or_init(|| Mutex::new(ThreadPool::new(1)))
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

pub fn track_process(cvat: &cvAutoTrack, sender: Option<Sender<WsEvent>>) -> Result<(), Box<dyn Error>> {
    let mut trackdata = TrackData::default();
    match track(
        &cvat,
        &mut trackdata.x,
        &mut trackdata.y,
        &mut trackdata.a,
        &mut trackdata.r,
        &mut trackdata.m,
    ) {
        Ok(_) => {}
        Err(e) => {
            trackdata.err = e.to_string();
            let _ = sender.unwrap().send(WsEvent::Track(trackdata));
            return Err(e);
        }
    }
    let _ = sender.unwrap().send(WsEvent::Track(trackdata));
    Ok(())
}

pub fn track(
    cvat: &cvAutoTrack,
    x: &mut c_double,
    y: &mut c_double,
    a: &mut c_double,
    r: &mut c_double,
    m: &mut c_int,
) -> Result<(), Box<dyn Error>> {
    if unsafe {!cvat.GetTransformOfMap(x, y, a, m)} {        
        return Err(get_last_err_json(&cvat).to_str().unwrap().into());
    }
    if unsafe {!cvat.GetRotation(r)} {
        return Err(get_last_err_json(&cvat).to_str().unwrap().into());
    }
    Ok(())
}

fn get_last_err_json(cvat: &cvAutoTrack) -> &CStr {
    let mut cs:[i8; 256] = [0; 256];
    let c_buf: *mut i8 = cs.as_mut_ptr();
    unsafe { cvat.GetLastErrJson(c_buf, 256) };
    unsafe { CStr::from_ptr(c_buf) }
}

fn ensure_cvat_instance() -> &'static Mutex<Option<(cvAutoTrack, Library)>> {
    CVAT_INSTANCE.get_or_init(|| Mutex::new(None))
}

pub fn initialize_cvat() -> Result<(), Box<dyn Error>> {
    let cvat = match set_lib_directory() {
        Ok(_) => unsafe { cvAutoTrack::new("cvAutoTrack.dll") }
            .expect("ERROR loading cvAutoTrack.dll"),
        Err(_) => unsafe { cvAutoTrack::new("./cvAutoTrack/cvAutoTrack.dll") }
            .expect("ERROR loading cvAutoTrack.dll"),
    };
    *ensure_cvat_instance().lock().unwrap() = Some((cvat, unsafe { Library::new("cvAutoTrack.dll").unwrap() }));
    Ok(())
}

pub fn unload_cvat() -> Result<(), Box<dyn Error>> {
    let mut guard = ensure_cvat_instance().lock().unwrap();
    if let Some((cvat, _)) = guard.take() {
        if get_is_tracking() {
            unsafe { cvat.uninit() };
            set_is_tracking(false);
        }
        cvat.close();
    }
    Ok(())
}