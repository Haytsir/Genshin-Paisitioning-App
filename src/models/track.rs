use libc::{c_double, c_int};
pub use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy)]
pub struct TrackData {
    pub x: c_double,
    pub y: c_double,
    pub a: c_double,
    pub r: c_double,
    pub m: c_int,
    pub err: &'static str,
}

impl Clone for TrackData {
    fn clone(&self) -> TrackData {
        *self
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Copy, Clone)]
pub enum CaptureMode {
    DirectX,
    Bitblt,
}
