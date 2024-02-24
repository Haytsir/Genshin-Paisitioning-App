#[cfg(test)]
mod lib_tests {
    use std::{error::Error, ffi::{c_double, c_int, CStr}, os::windows::ffi::{OsStrExt, OsStringExt}, path::PathBuf};
    use crate::{cvat::cvAutoTrack, models};
    use windows::Win32::System::LibraryLoader::SetDllDirectoryW;

    #[test]
    fn libload() {
        let mut d: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("lib\\bin");

        let mut dll_dir_vec = d.to_str().expect("Unexpected directory name").encode_utf16().collect::<Vec<_>>();
        dll_dir_vec.push(0);
        let dll_dir = dll_dir_vec.as_ptr() as *mut u16;
        
        unsafe { let _ = SetDllDirectoryW( windows::core::PCWSTR::from_raw(dll_dir) ); };

        println!("{}", d.display());
        let cvat = unsafe { cvAutoTrack::new("cvAutoTrack.dll").expect("cvAutoTrack.dll load failed.") };


        let mut cs:[i8; 256] = [0; 256];
        let c_buf: *mut i8 = cs.as_mut_ptr();
        unsafe {
            cvat.GetCompileVersion(c_buf, 256);
        }
        let mut c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
        let mut str_slice: &str = c_str.to_str().unwrap(); // .to_owned() if want to own the str.
        println!("Compile Version: {}", str_slice);

        unsafe {
            cvat.GetCompileTime(c_buf, 256);
        }
        c_str = unsafe { CStr::from_ptr(c_buf) };
        str_slice = c_str.to_str().unwrap(); // .to_owned() if want to own the str.
        println!("Compile Time: {}", str_slice);

        unsafe{cvat.InitResource()};
        if unsafe{cvat.DebugLoadMapImagePath("map.jpg".as_ptr() as *const i8)} {
            println!("DebugLoadMapImagePath: OK");
        } else {
            println!("DebugLoadMapImagePath: NG");
            let mut cs:[i8; 256] = [0; 256];
            let c_buf: *mut i8 = cs.as_mut_ptr();
            unsafe { cvat.GetLastErrJson(c_buf, 256) };
            let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
            println!("{}", c_str.to_str().unwrap());
        }
        
        for _ in 0..10 { 
            let _ = track_process(&cvat);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        };
        std::thread::sleep(std::time::Duration::from_millis(5000));
        drop(cvat);
        
    }

    pub fn track_process(cvat: &cvAutoTrack) -> Result<(), Box<dyn Error>> {
        let mut trackdata: models::TrackData = Default::default();
        match track(
            cvat,
            &mut trackdata.x,
            &mut trackdata.y,
            &mut trackdata.a,
            &mut trackdata.r,
            &mut trackdata.m,
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("track_process: {}", e);
                
                //trackdata.err = e.to_string().try_into();
                return Err(e);
            }
        }
        println!("trackdata: {:?}", trackdata);
        Ok(())
    }

    pub fn track(
        cvat: &cvAutoTrack,
        x: &mut c_double,
        y: &mut c_double,
        a: &mut c_double,
        r: &mut c_double,
        m: &mut c_int,
    ) -> Result<(), Box<dyn Error>> {
        log::info!("track");
        
        if unsafe {!cvat.GetTransformOfMap(x, y, a, m)} {
            println!("track:GetTransformOfMap");
            
            let mut cs:[i8; 256] = [0; 256];
            let c_buf: *mut i8 = cs.as_mut_ptr();
            unsafe { cvat.GetLastErrJson(c_buf, 256) };
            let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
            return Err(c_str.to_str().unwrap().into());
        }
        if unsafe {!cvat.GetRotation(r)} {
            println!("track:GetRotation");

            let mut cs:[i8; 256] = [0; 256];
            let c_buf: *mut i8 = cs.as_mut_ptr();
            unsafe { cvat.GetLastErrJson(c_buf, 256) };
            let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
            return Err(c_str.to_str().unwrap().into());
        }
        Ok(())
    }
}