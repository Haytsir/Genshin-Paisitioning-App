mod config_handler;
mod update_handler;
mod track_handler;

pub use config_handler::*;
pub use update_handler::*;
pub use track_handler::*;

#[derive(Debug)]
pub enum CustomError {
    UPDATE_FAILED,
    CONFIG_ERROR,
}

impl warp::reject::Reject for CustomError {} 