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

// 여러 다른 모듈에서 ? 연산자로 에러를 전파할 때 적절한 에러 타입으로 변환하기 위함.
impl From<Box<dyn Error + Send + Sync>> for CvatError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        if error.is::<std::io::Error>() {
            CvatError::InitializationError(error.to_string())
        } else if error.to_string().contains("track") {
            CvatError::TrackingError(error.to_string())
        } else if error.to_string().contains("lock") {
            CvatError::LockError(error.to_string())
        } else {
            CvatError::LibraryError(error.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, CvatError>;
pub type StdResult<T> = std::result::Result<T, Box<dyn Error>>; 