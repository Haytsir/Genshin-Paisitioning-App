use crossbeam_channel::Sender;
use warp::reject::Rejection;
use crate::models::WsEvent;
use crate::cvat::{unload_cvat, start_track_thead};

pub struct TrackHandler;

impl TrackHandler {
    pub fn handle_init(tx: &Option<Sender<WsEvent>>, use_bit_blt: bool) -> Result<(), Rejection> {
        start_track_thead(tx.as_ref().cloned(), use_bit_blt);
        Ok(())
    }

    pub fn handle_uninit() -> Result<(), Rejection> {
        let _ = unload_cvat();
        Ok(())
    }
} 