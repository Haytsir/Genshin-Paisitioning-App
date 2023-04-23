extern crate libc;
extern crate libloading;

use libc::{c_char, c_double, c_int};

type FuncParamVoidRetBool = unsafe extern "C" fn() -> bool;
type FuncGetTransformOfMap =
    unsafe extern "C" fn(*mut c_double, *mut c_double, *mut c_double, *mut c_int) -> bool;
type FuncGetRotation = unsafe extern "C" fn(*mut c_double) -> bool;
type FuncCharBuffRetInt = unsafe extern "C" fn(*const c_char, c_int) -> c_int;
type FuncCharBuffRetBool = unsafe extern "C" fn(*const c_char, c_int) -> bool;
// https://rinthel.github.io/rust-lang-book-ko/ch10-03-lifetime-syntax.html
// https://users.rust-lang.org/t/multiple-objects-with-interdependent-lifetimes-in-the-same-struct/16507/4
#[derive(Copy, Clone, Debug)]
pub struct LibCvat {
    func_init: FuncParamVoidRetBool,
    func_uninit: FuncParamVoidRetBool,
    func_set_use_bitblt_capture_mode: FuncParamVoidRetBool,
    func_set_use_dx11_capture_mode: FuncParamVoidRetBool,
    func_get_transform_of_map: FuncGetTransformOfMap,
    func_get_rotation: FuncGetRotation,
    func_get_last_err_json: FuncCharBuffRetInt,
    func_set_disable_file_log: FuncParamVoidRetBool,
    func_get_compile_version: FuncCharBuffRetBool,
    func_get_compile_time: FuncCharBuffRetBool,
}

impl<'lib> LibCvat {
    pub fn new(lib: super::LIB) -> LibCvat {
        unsafe {
            let func_init = *lib.get(b"init").unwrap();
            let func_uninit = *lib.get(b"uninit").unwrap();
            let func_set_use_bitblt_capture_mode = *lib.get(b"SetUseBitbltCaptureMode").unwrap();
            let func_set_use_dx11_capture_mode = *lib.get(b"SetUseDx11CaptureMode").unwrap();
            let func_get_transform_of_map = *lib.get(b"GetTransformOfMap").unwrap();
            let func_get_rotation = *lib.get(b"GetRotation").unwrap();
            let func_get_last_err_json = *lib.get(b"GetLastErrJson").unwrap();
            let func_set_disable_file_log = *lib.get(b"SetDisableFileLog").unwrap();
            let func_get_compile_version = *lib.get(b"GetCompileVersion").unwrap();
            let func_get_compile_time = *lib.get(b"GetCompileTime").unwrap();
            LibCvat {
                func_init,
                func_uninit,
                func_set_use_bitblt_capture_mode,
                func_set_use_dx11_capture_mode,
                func_get_transform_of_map,
                func_get_rotation,
                func_get_last_err_json,
                func_set_disable_file_log,
                func_get_compile_version,
                func_get_compile_time,
            }
        }
    }
    pub fn init(&self) -> bool {
        unsafe { (self.func_init)() }
    }
    pub fn uninit(&self) -> bool {
        unsafe { (self.func_uninit)() }
    }
    pub fn set_use_bitblt_capture_mode(&self) -> bool {
        unsafe { (self.func_set_use_bitblt_capture_mode)() }
    }
    pub fn set_use_dx11_capture_mode(&self) -> bool {
        unsafe { (self.func_set_use_dx11_capture_mode)() }
    }
    pub fn get_transform_of_map(
        &self,
        x: *mut c_double,
        y: *mut c_double,
        a: *mut c_double,
        map_id: *mut c_int,
    ) -> bool {
        unsafe { (self.func_get_transform_of_map)(x, y, a, map_id) }
    }
    pub fn get_rotation(&self, a: *mut c_double) -> bool {
        unsafe { (self.func_get_rotation)(a) }
    }
    pub fn get_last_err_json(&self, json_buff: *const c_char, buff_size: c_int) -> c_int {
        unsafe { (self.func_get_last_err_json)(json_buff, buff_size) }
    }
    pub fn set_disable_file_log(&self) -> bool {
        unsafe { (self.func_set_disable_file_log)() }
    }
    pub fn get_compile_version(&self, time_buff: *const c_char, buff_size: c_int) -> bool {
        unsafe { (self.func_get_compile_version)(time_buff, buff_size) }
    }
    pub fn get_compile_time(&self, time_buff: *const c_char, buff_size: c_int) -> bool {
        unsafe { (self.func_get_compile_time)(time_buff, buff_size) }
    }
}

pub trait Cvat {
    fn init(&mut self) -> bool;
    fn uninit(&self) -> bool;
    fn start_serve(&self) -> bool;
    fn stop_serve(&self) -> bool;
    fn set_use_bitblt_capture_mode(&self) -> bool;
    fn set_use_dx11_capture_mode(&self) -> bool;
    fn get_transform_of_map(
        &self,
        x: *mut c_double,
        y: *mut c_double,
        a: *mut c_double,
        map_id: *mut c_int,
    ) -> bool;
    fn get_rotation(&self, a: *mut c_double) -> bool;
    fn get_last_err_json(&self, json_buff: *const c_char, buff_size: c_int) -> c_int;
    fn set_disable_file_log(&self) -> bool;
    fn set_enable_file_log(&self) -> bool;
    fn get_compile_version(&self, time_buff: *const c_char, buff_size: c_int) -> bool;
    fn get_compile_time(&self, time_buff: *const c_char, buff_size: c_int) -> bool;
}
