use config::Config;
use crossbeam_channel::Sender;
use warp::reject::Rejection;
use crate::models::WsEvent;
use crate::app::updater;

pub struct UpdateHandler;

impl UpdateHandler {
    pub fn handle_app_update(
        id: String, 
        force: bool, 
        config: &Config, 
        tx: &Option<Sender<WsEvent>>
    ) -> Result<(), Rejection> {
        updater::check_app_update(config.clone(), id, tx.clone(), force)
            .map_err(|_| warp::reject::custom(super::CustomError::UPDATE_FAILED))?;
        Ok(())
    }

    pub fn handle_lib_update(
        id: String, 
        force: bool, 
        config: &Config, 
        tx: &Option<Sender<WsEvent>>
    ) -> Result<(), Rejection> {
        updater::check_lib_update(config.clone(), id, tx.clone(), force)
            .map_err(|_| warp::reject::custom(super::CustomError::UPDATE_FAILED))?;
        Ok(())
    }
} 