use std::sync::Mutex;
use once_cell::sync::OnceCell;
use libloading::Library;
use super::bindings::cvAutoTrack;
use threadpool::ThreadPool;
use std::path::PathBuf;

pub struct CvatState {
    pub is_tracking: bool,
    pub capture_interval: u64,
    pub capture_delay_on_error: u64,
    pub instance: Option<(cvAutoTrack, Library)>,
    pub thread_pool: ThreadPool,
    dll_path: PathBuf,
}

impl Default for CvatState {
    fn default() -> Self {
        Self {
            is_tracking: false,
            capture_interval: 250,
            capture_delay_on_error: 800,
            instance: None,
            thread_pool: ThreadPool::new(1),
            dll_path: PathBuf::new(),
        }
    }
}

impl CvatState {
    pub fn set_tracking(&mut self, value: bool) {
        self.is_tracking = value;
    }

    pub fn get_thread_pool(&self) -> ThreadPool {
        self.thread_pool.clone()
    }
}

static STATE: OnceCell<Mutex<CvatState>> = OnceCell::new();

pub fn get_state() -> &'static Mutex<CvatState> {
    STATE.get_or_init(|| Mutex::new(CvatState::default()))
} 