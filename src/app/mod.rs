pub mod config;
pub mod installer;
pub mod updater;
pub mod path;

mod tray;
mod utils;

pub use self::tray::*;
pub use self::utils::*;

/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */

use once_cell::sync::Lazy;
use crate::models::AppState;

static APP_STATE: Lazy<AppState> = Lazy::new(AppState::new);

pub fn get_app_state() -> &'static AppState {
    &APP_STATE
}
