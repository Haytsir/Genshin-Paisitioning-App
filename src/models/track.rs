use libc::{c_double, c_int};
pub use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TrackData {
    pub x: c_double,
    pub y: c_double,
    pub a: c_double,
    pub r: c_double,
    pub m: c_int,
    pub err: String,
}

impl Clone for TrackData {
    fn clone(&self) -> TrackData {
        TrackData {
            x: self.x,
            y: self.y,
            a: self.a,
            r: self.r,
            m: self.m,
            err: self.err.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Copy, Clone)]
pub enum CaptureMode {
    DirectX,
    Bitblt,
}
