pub mod config;
pub mod installer;
mod tray;
pub mod updater;
mod utils;

pub use self::tray::*;
pub use self::utils::*;

/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */
