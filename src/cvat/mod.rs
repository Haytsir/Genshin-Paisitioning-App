/*
 * 일반적인 pub mod features;로 사용하게 되면,
 * implement함수 접근 시 cvat::features::함수명 으로 접근해야함.
 * 이를 inline module로 둠으로써, cvat::함수명 으로 접근 가능하도록 한다.
 */
mod bindings;
mod error;
mod state;
mod tracking;
mod translations;
mod features;

pub use error::{CvatError, Result};
pub use tracking::Tracker;
pub use features::*;

use crate::app::path::get_lib_path;
use state::get_state;

pub fn initialize_cvat() -> Result<()> {
    let mut state = get_state().lock().unwrap();
    let cvat = unsafe { 
        bindings::cvAutoTrack::new(get_lib_path().join("cvAutoTrack.dll").to_str().unwrap())
    }.map_err(|e| CvatError::LibraryError(e.to_string()))?;

    let lib = unsafe {
        libloading::Library::new(get_lib_path().join("cvAutoTrack.dll"))
    }.map_err(|e| CvatError::LibraryError(e.to_string()))?;

    state.instance = Some((cvat, lib));
    Ok(())
}

pub fn unload_cvat() -> Result<()> {
    let mut state = get_state().lock().unwrap();
    if let Some((cvat, _)) = state.instance.take() {
        if state.is_tracking {
            unsafe { cvat.uninit() };
            state.is_tracking = false;
        }
        cvat.close();
    }
    Ok(())
}

pub fn set_capture_interval(interval: u64) {
    get_state().lock().unwrap().capture_interval = interval.max(100);
}

pub fn set_capture_delay_on_error(delay: u64) {
    get_state().lock().unwrap().capture_delay_on_error = delay.max(100);
}

pub fn get_is_tracking() -> bool {
    get_state().lock().unwrap().is_tracking
}

#[cfg(test)]
mod test;
/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */
