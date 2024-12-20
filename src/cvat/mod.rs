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

use crate::models::{SendEvent, WsEvent};
use crate::{app::path::get_lib_path};
use crate::events::{EventBus};
use std::error::Error;
use crate::websocket::WebSocketHandler;
use std::sync::Arc;
use crate::app::get_app_state;

pub fn initialize_cvat() -> Result<()> {
    log::debug!("Initialize Cvat");
    let mut state = get_app_state();
    
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
    state.set_instance(None);
    Ok(())
}

pub async fn register_events(
    event_bus: &Arc<EventBus>, 
    ws_handler: &Arc<WebSocketHandler>
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    log::debug!("Register Cvat Events");
    let event_bus1 = event_bus.clone();
    let ws_handler1 = ws_handler.clone();
    ws_handler.register("init", move |id, _| {
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

    ws_handler.register("uninit", |id, _| async move {
        unload_cvat().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(())
    }).await?;

    // ... 다른 이벤트 핸들러들
    Ok(())
}

#[cfg(test)]
mod test;
/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */
