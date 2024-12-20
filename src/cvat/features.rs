use super::Tracker;
use crate::{events::EventBus, models::WsEvent, websocket::WebSocketHandler};
use config::Config;
use crate::models::AppConfig;
use crate::app::get_app_state;
use std::sync::Arc;

pub fn start_track_thread(event_bus: Arc<EventBus>, ws_handler: Arc<WebSocketHandler>) -> bool {
    log::debug!("Start Track Thread");
    if super::initialize_cvat().is_err() {
        return false;
    }
    let state = get_app_state();
    let instance = state.get_instance();
    if let Some(cvat) = instance.as_ref() {
        log::debug!("Cvat Instance Found");
        let tracker = Tracker::new(cvat);
        match tracker.start(ws_handler) {
            Ok(_) => {
                log::debug!("Track Thread Ready");
                true
            }
            Err(_) => false,
        }
    } else {
        log::debug!("No Cvat Instance");
        false
    }
}