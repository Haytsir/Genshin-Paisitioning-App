use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CvatError {
    InitializationError(String),
    TrackingError(String),
    LibraryError(String),
}

impl fmt::Display for CvatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            Self::TrackingError(msg) => write!(f, "Tracking error: {}", msg),
            Self::LibraryError(msg) => write!(f, "Library error: {}", msg),
        }
    }
}

impl Error for CvatError {}

pub type Result<T> = std::result::Result<T, CvatError>; 