use super::Tracker;
use crossbeam_channel::Sender;
use crate::models::WsEvent;
use config::Config;
use crate::models::AppConfig;
use super::state::get_state;

pub fn start_track_thead(sender: Option<Sender<WsEvent>>, use_bit_blt: bool) -> bool {
    let state = get_state().lock().unwrap();
    if let Some((cvat, _)) = &state.instance {
        let tracker = Tracker::new(cvat);
        match tracker.start(sender, use_bit_blt) {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        false
    }
}

pub fn stop_track_thread() -> bool {
    Tracker::stop().is_ok()
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
    super::set_capture_interval(u64::from(new_app_config.capture_interval));
    super::set_capture_delay_on_error(u64::from(new_app_config.capture_delay_on_error));
    
    let _ = crate::app::config::save_config(&new_app_config);
}

pub use super::{
    initialize_cvat,
    unload_cvat,
    set_capture_interval,
    set_capture_delay_on_error,
    get_is_tracking,
};