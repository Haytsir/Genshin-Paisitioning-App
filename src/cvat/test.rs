#[cfg(test)]
mod lib_tests {
    use std::{path::PathBuf, ffi::{CStr, c_double, c_int}, error::Error};

    use crate::{cvat::cvAutoTrack, models};

    #[test]
    fn libload() {
        let mut d: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("lib/bin/cvAutoTrack.dll");
        println!("{}", d.display());
        
        let cvat = unsafe {cvAutoTrack::new(d.as_os_str()).expect("cvAutoTrack.dll load failed.")};

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
        
        for _ in 0..10 { 
            let _ = track_process(&cvat);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        };
        std::thread::sleep(std::time::Duration::from_millis(5000));
        drop(cvat);
        //assert_eq!(2 + 2, 4);
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