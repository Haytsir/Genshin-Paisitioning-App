use super::error::*;
use super::translations::translate_error_json;
use super::bindings::cvAutoTrack;
use crate::app::get_app_state;
use crate::models::{SendEvent, TrackData, WsEvent};
use crate::websocket::WebSocketHandler;
use std::thread;
use std::time::Duration;
use libc::{c_double, c_int};
use warp::filters::ws;
use std::ffi::CStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::runtime::Runtime;
use tokio::spawn;

pub struct Tracker<'a> {
    cvat: &'a cvAutoTrack,
}

impl<'a> Tracker<'a> {
    pub fn new(cvat: &'a cvAutoTrack) -> Self {
        Self { 
            cvat
        }
    }

    pub fn start(&self, ws_handler: Arc<WebSocketHandler>) -> Result<()> {
        log::debug!("Start Track");

        let state = get_app_state();
        state.set_tracking(true);
        
        // Arc<AtomicBool|AtomicU32>로 선언. 이는 여러 스레드에서 공유되는 동일한 값을 가리킨다.
        // 이를 통해 여러 스레드에서 동일한 값을 읽고 쓰는 것을 보장한다.
        // state.set_tracking(false)를 호출하면 내부적으로 동일한 Arc<AtomicBool>을 업데이트함.
        // Arc::clone(&is_tracking)으로 얻은 참조는 동일한 AtomicBool을 가리키는 새로운 Arc 핸들을 생성함.
        let interval = Arc::clone(&state.capture_interval);
        let delay = Arc::clone(&state.capture_delay_on_error);
        let is_tracking = Arc::clone(&state.is_tracking);
        
        let cvat = unsafe { &*(self.cvat as *const _) };
        let ws_handler_thread = ws_handler.clone();
        
        // spawn_blocking을 사용하여 별도 스레드에서 실행
        tokio::task::spawn_blocking(move || {
            let rt = Runtime::new().unwrap();
            log::debug!("Tracking Thread Started");
            
            while is_tracking.load(Ordering::Relaxed) {
                let mut trackdata = TrackData::default();
                match Tracker::track(cvat, &mut trackdata.x, &mut trackdata.y, &mut trackdata.a, 
                    &mut trackdata.r, &mut trackdata.m) {
                    Ok(_) => {
                        rt.block_on(ws_handler_thread.broadcast(
                            SendEvent::from(WsEvent::Track { data: trackdata })
                        ));
                        thread::sleep(Duration::from_millis(
                            interval.load(Ordering::Relaxed).into()
                        ));
                    },
                    Err(e) => {
                        rt.block_on(ws_handler_thread.broadcast(
                            SendEvent::from(WsEvent::Track { data: trackdata })
                        ));
                        thread::sleep(Duration::from_millis(
                            delay.load(Ordering::Relaxed).into()
                        ));
                    }
                }
            }
            unsafe { cvat.uninit() };
            state.set_tracking(false);
            log::debug!("Tracking Thread Stopped");
            rt.block_on(ws_handler_thread.broadcast(SendEvent::from(WsEvent::Uninit {})));
        });
        
        Ok(())
    }

    fn track(
        cvat: &cvAutoTrack,
        x: &mut c_double,
        y: &mut c_double,
        a: &mut c_double,
        r: &mut c_double,
        m: &mut c_int,
    ) -> Result<()> {
        if !unsafe { cvat.GetTransformOfMap(x, y, a, m) } {
            return Err(CvatError::TrackingError(Tracker::get_last_error(cvat)));
        }
        if unsafe { !cvat.GetRotation(r) } {
            return Err(CvatError::TrackingError(Tracker::get_last_error(cvat)));
        }
        Ok(())
    }

    fn get_last_error(cvat: &cvAutoTrack) -> String {
        let mut cs: [i8; 256] = [0; 256];
        let c_buf: *mut i8 = cs.as_mut_ptr();
        unsafe { cvat.GetLastErrJson(c_buf, 256) };
        let error_json = unsafe { CStr::from_ptr(c_buf) }.to_str().unwrap_or("{}");
        translate_error_json(error_json).unwrap_or_else(|_| error_json.to_string())
    }
} 