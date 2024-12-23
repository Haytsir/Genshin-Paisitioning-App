/*
 * 일반적인 pub mod features;로 사용하게 되면,
 * implement함수 접근 시 cvat::features::함수명 으로 접근해야함.
 * 이를 inline module로 둠으로써, cvat::함수명 으로 접근 가능하도록 한다.
 */
pub mod bindings;
mod error;
mod tracking;
mod translations;
mod features;

pub use error::{CvatError, Result};
pub use tracking::Tracker;
pub use features::*;

use crate::models::{AppEvent, SendEvent, WsEvent};
use crate::app::path::get_lib_path;
use crate::events::EventBus;
use std::error::Error;
use std::ffi::CStr;
use crate::websocket::WebSocketHandler;
use std::sync::Arc;
use crate::app::get_app_state;

pub fn is_cvat_loaded() -> bool {
    let state = get_app_state();
    state.get_instance().is_some()
}

pub fn initialize_cvat() -> Result<()> {
    log::debug!("Initialize Cvat");
    let state = get_app_state();
    
    // 이미 인스턴스가 있다면 그대로 사용
    if state.get_instance().is_some() {
        return Ok(());
    }

    let cvat = unsafe { 
        bindings::cvAutoTrack::new(get_lib_path().join("cvAutoTrack.dll").to_str().unwrap())
    }.map_err(|e| CvatError::LibraryError(e.to_string()))?;

    state.set_instance(Some(cvat));
    Ok(())
}

pub fn unload_cvat() -> Result<()> {
    let state = get_app_state();
    log::debug!("Unloading CVAT...");
    
    // instance를 먼저 가져와서 Option<cvAutoTrack>으로 소유권 이전
    let mut instance = state.get_instance();
    if let Some(cvat) = instance.as_ref() {
        unsafe { cvat.close(); }
    }
    
    // instance를 None으로 설정
    drop(instance);  // 명시적으로 읽기 lock 해제
    state.set_instance(None);
    
    log::debug!("CVAT unloaded successfully");
    Ok(())
}

pub async fn register_events(
    event_bus: &Arc<EventBus>, 
    ws_handler: &Arc<WebSocketHandler>
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    log::debug!("Register Cvat Events");
    let event_bus1 = event_bus.clone();
    let ws_handler1 = ws_handler.clone();
    ws_handler.register("init", move |_, _| {
        let event_bus = event_bus1.clone();
        let ws_handler = ws_handler1.clone();
        async move {
            initialize_cvat().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            if start_track_thread(event_bus.clone(), ws_handler.clone()) {
                ws_handler.broadcast(SendEvent::from(WsEvent::DoneInit)).await?;
            }
            Ok(())
        }
    }).await?;
    
    let event_bus2 = event_bus.clone();
    ws_handler.register("uninit", move |_, _| {
        let event_bus = event_bus2.clone();
        async move {
            unload_cvat().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            event_bus.emit(&AppEvent::DoneUninit()).await.unwrap();
            Ok(())
        }
    }).await?;

    let event_bus3 = event_bus.clone();
    event_bus.register(AppEvent::Uninit(), move |_event| {
        let event_bus = event_bus3.clone();
        async move {
            log::debug!("Uninit Event");
            unload_cvat().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            event_bus.emit(&AppEvent::DoneUninit()).await.unwrap();
            Ok(())
        }
    }).await?;

    // ... 다른 이벤트 핸들러들
    Ok(())
}

pub fn get_cvat_version() -> String {
    let state = get_app_state();
    let instance = state.get_instance();
    if let Some(cvat) = instance.as_ref() {
        let mut c_buf: [i8; 256] = [0; 256];
        unsafe { cvat.GetCompileVersion(c_buf.as_mut_ptr(), 256); }
        let c_str: &CStr = unsafe { CStr::from_ptr(c_buf.as_ptr()) };
        let str_slice: &str = c_str.to_str().unwrap();
        return str_slice.to_string();
    }
    return String::new();
}

#[cfg(test)]
mod test;
/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */
