/*
 * 일반적인 pub mod features;로 사용하게 되면,
 * implement함수 접근 시 cvat::features::함수명 으로 접근해야함.
 * 이를 inline module로 둠으로써, cvat::함수명 으로 접근 가능하도록 한다.
 */
mod features;
mod bindings;

pub use self::features::*;

#[cfg(test)]
mod test;
/*
https://stackoverflow.com/questions/66313302/rust-ffi-include-dynamic-library-in-cross-platform-fashion
https://stackoverflow.com/questions/66252029/how-to-dynamically-call-a-function-in-a-shared-library
 */
