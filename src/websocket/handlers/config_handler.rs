use config::Config;
use crossbeam_channel::Sender;
use warp::reject::Rejection;
use crate::models::{WsEvent, AppConfig};
use crate::cvat::on_config_changed;

pub struct ConfigHandler;

impl ConfigHandler {
    pub fn handle_get_config(id: String, config: &Config, tx: &Option<Sender<WsEvent>>) -> Result<(), Rejection> {
        if let Some(t) = tx.as_ref() {
            let app_config: AppConfig = config.clone().try_deserialize()
                .map_err(|_| warp::reject::custom(super::CustomError::CONFIG_ERROR))?;
            t.send(WsEvent::Config(app_config, id)).unwrap();
        }
        Ok(())
    }

    pub fn handle_set_config(
        mut new_app_config: AppConfig, 
        id: String, 
        config: &mut Config, 
        tx: &Option<Sender<WsEvent>>
    ) -> Result<(), Rejection> {
        if let Some(t) = tx.as_ref() {
            let new_config = Config::builder()
                .add_source(config.clone())
                .set_override("captureInterval", new_app_config.capture_interval)
                .and_then(|c| c.set_override("captureDelayOnError", new_app_config.capture_delay_on_error))
                .and_then(|c| c.set_override("useBitBltCaptureMode", new_app_config.use_bit_blt_capture_mode))
                .and_then(|c| c.build())
                .map_err(|_| warp::reject::custom(super::CustomError::CONFIG_ERROR))?;

            on_config_changed(config.clone(), new_config.clone());
            *config = new_config;
            
            new_app_config.changed = Some(true);
            t.send(WsEvent::Config(new_app_config, id)).unwrap();
        }
        Ok(())
    }
} 