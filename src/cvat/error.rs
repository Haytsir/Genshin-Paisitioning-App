use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CvatError {
    InitializationError(String),
    TrackingError(String),
    LibraryError(String),
    LockError(String),
}

impl fmt::Display for CvatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            Self::TrackingError(msg) => write!(f, "Tracking error: {}", msg),
            Self::LibraryError(msg) => write!(f, "Library error: {}", msg),
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
        }
    }
}

impl Error for CvatError {}

impl From<Box<dyn Error + Send + Sync>> for CvatError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        CvatError::LibraryError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CvatError>;
pub type StdResult<T> = std::result::Result<T, Box<dyn Error>>; 