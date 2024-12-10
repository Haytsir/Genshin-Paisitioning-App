use super::{state::*, error::*};
use super::translations::translate_error_json;
use super::bindings::cvAutoTrack;
use crossbeam_channel::Sender;
use crate::models::{TrackData, WsEvent};
use std::thread;
use std::time::Duration;
use libc::{c_double, c_int};
use std::ffi::CStr;

pub struct Tracker<'a> {
    cvat: &'a cvAutoTrack,
}

impl<'a> Tracker<'a> {
    pub fn new(cvat: &'a cvAutoTrack) -> Self {
        Self { cvat }
    }

    pub fn start(&self, sender: Option<Sender<WsEvent>>, use_bit_blt: bool) -> Result<()> {
        let mut state = get_state().lock().unwrap();
        if state.is_tracking {
            return Ok(());
        }

        if unsafe { !self.cvat.init() } {
            return Err(CvatError::InitializationError("Failed to initialize tracking".into()));
        }

        state.is_tracking = true;
        let thread_pool = state.thread_pool.clone();
        drop(state);
        let cvat = unsafe { &*(self.cvat as *const _) };
        thread_pool.execute(move || {
            while get_state().lock().unwrap().is_tracking {
                match Self::track_process(cvat, sender.clone()) {
                    Ok(_) => thread::sleep(Duration::from_millis(
                        get_state().lock().unwrap().capture_interval
                    )),
                    Err(_) => thread::sleep(Duration::from_millis(
                        get_state().lock().unwrap().capture_delay_on_error
                    )),
                }
            }
            unsafe { cvat.uninit() };
        });
        
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if let Ok(mut state) = get_state().lock() {
            state.is_tracking = false;
        }
        Ok(())
    }

    fn track_process(cvat: &cvAutoTrack, sender: Option<Sender<WsEvent>>) -> Result<()> {
        let mut trackdata = TrackData::default();
        match Self::track(
            cvat,
            &mut trackdata.x,
            &mut trackdata.y,
            &mut trackdata.a,
            &mut trackdata.r,
            &mut trackdata.m,
        ) {
            Ok(_) => {
                if let Some(tx) = sender {
                    let _ = tx.send(WsEvent::Track(trackdata));
                }
                Ok(())
            }
            Err(e) => {
                trackdata.err = e.to_string();
                if let Some(tx) = sender {
                    let _ = tx.send(WsEvent::Track(trackdata));
                }
                Err(e)
            }
        }
    }

    fn track(
        cvat: &cvAutoTrack,
        x: &mut c_double,
        y: &mut c_double,
        a: &mut c_double,
        r: &mut c_double,
        m: &mut c_int,
    ) -> Result<()> {
        if unsafe { !cvat.GetTransformOfMap(x, y, a, m) } {
            return Err(CvatError::TrackingError(Self::get_last_error(cvat)));
        }
        if unsafe { !cvat.GetRotation(r) } {
            return Err(CvatError::TrackingError(Self::get_last_error(cvat)));
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