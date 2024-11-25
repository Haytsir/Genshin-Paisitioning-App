use crate::app::path;
use std::fs;
use std::{ffi::CString, path::PathBuf};
use std::path::Path;
use std::process::Command;
use log::debug;
use windows::{
    core::{s, Result as WinResult, PCSTR}, Win32::Foundation::*, Win32::Security::*, Win32::{System::Memory::*, UI::Shell::ShellExecuteA},
    Win32::System::{
                    LibraryLoader::SetDllDirectoryW,
                    Threading::*
    },
};


pub fn set_lib_directory() -> Result<(), std::io::Error> {
    let mut d: PathBuf;

    d = PathBuf::from(std::env::current_exe().unwrap());
    d.pop();
    d.push("cvAutoTrack");

    log::debug!("{}", d.display());

    #[cfg(debug_assertions)]
    {
        let mut dd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dd.push("lib\\bin");

        match fs::metadata(&dd) {
            Ok(_) => d = dd,
            Err(_) => log::debug!("lib\\bin doesn't exist. Using current directory."),
        }
    }

    log::debug!("{}", d.display());

    match fs::metadata(&d) {
        Ok(_) => {
            let mut dll_dir_vec = d.to_str().expect("Unexpected directory name").encode_utf16().collect::<Vec<_>>();
            dll_dir_vec.push(0);
            let dll_dir = dll_dir_vec.as_ptr() as *mut u16;
            
            unsafe { let _ = SetDllDirectoryW( windows::core::PCWSTR::from_raw(dll_dir) ); };
            return Ok(());
        }
        Err(e) => {
            log::debug!("Library Directory: {}", e);
            return Err(e.into());
        }
    }
}


// 현재 프로그램이 프로젝트 디렉토리에서 실행중인지 확인한다.
pub fn check_proj_directory() -> Result<bool, std::io::Error> {
    // target_dir의 내용이 프로젝트 디렉토리의 Root가 된다.
    let target_dir = path::get_app_path();
    log::debug!("Project Directory: {}", target_dir.display());
    // 먼저 프로젝트 디렉토리가 존재하지 않는다면 생성한다.
    match std::fs::create_dir_all(path::get_cache_path()) {
        Ok(_) => {
            log::debug!("Project Directory: 생성 성공");
        }
        Err(e) => {
            log::debug!("Project Directory: 생성 실패");
            log::debug!("Error: {}", e);
            return Err(e);
        }
    }

    // Current Working Directory를 얻어낸다.
    let cwd = path::get_current_work_directory();

    // 현재 작업 디렉토리와 프로젝트 디렉토리를 대조한다.
    if !cwd.eq(&target_dir) {
        return Ok(false);
    } else {
        return Ok(true);
    }
}

pub fn check_elevation(target: &Path, args: Vec<String>) -> bool {
    if let Ok(elevated) = is_elevated() {
        if elevated {
            return true;
        }
    }
    run_shell_execute(target, args, Some(true));
    false
}

pub fn run_shell_execute(target: &Path, args: Vec<String>, with_elevation: Option<bool>) {
    let path_string = CString::new(target.to_str().unwrap()).unwrap();
    let path_ptr = CString::as_bytes_with_nul(&path_string);
    let path = PCSTR::from_raw(&(path_ptr[0]) as &u8 as *const u8);
    let arg_string = CString::new(args.join(" ")).unwrap();
    let arg_ptr = CString::as_bytes_with_nul(&arg_string);
    let arg = PCSTR::from_raw(&(arg_ptr[0]) as &u8 as *const u8);
    
    let mut elev = None;
    if with_elevation.unwrap_or(false) {
        elev = Some(s!("runas"));
    }
    unsafe {
        ShellExecuteA(
            None,
            elev.unwrap(),
            path,
            arg,
            None,
            windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD(0),
        );
    }
}

pub fn is_elevated() -> WinResult<bool> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut bytes_required = 0;
        _ = GetTokenInformation(token, TokenPrivileges, None, 0, &mut bytes_required);

        let buffer = LocalAlloc(LPTR, bytes_required as usize)?;

        GetTokenInformation(
            token,
            TokenPrivileges,
            Some(buffer.0 as *mut _),
            bytes_required,
            &mut bytes_required,
        )?;

        let header = &*(buffer.0 as *const TOKEN_PRIVILEGES);

        let privileges =
            std::slice::from_raw_parts(header.Privileges.as_ptr(), header.PrivilegeCount as usize);
        
        // SE_BACKUP_PRIVILEGE, SE_RESTORE_PRIVILEGE가 없다면 추가한다.
        let mut required_privileges: Vec<u32> = vec![0x11, 0x12];
        for privilege in privileges {
            for (i, p) in required_privileges.iter().enumerate() {
                if *p == privilege.Luid.LowPart {
                    required_privileges.remove(i);
                    break;
                }
            }
        }

        _ = LocalFree(buffer);
        Ok(required_privileges.len() == 0)
    }
}

pub fn run_cmd(cmd: &str) {
    let _ = Command::new("powershell")
        .args([
            "-Command",
            "Start-Process",
            "-FilePath",
            "cmd",
            "-ArgumentList",
            format!("\"/C {cmd}\"").as_str(),
        ])
        .status();
}

pub fn is_process_already_running() -> bool {
    log::debug!("Checking if process already running...");
    let instance = single_instance::SingleInstance::new(env!("CARGO_PKG_NAME")).unwrap();
    if instance.is_single() {
        log::debug!("No other instance is running.");
    } else {
        log::debug!("Another instance is already running.");
    }
    !instance.is_single()
}

pub fn terminate_process() {
    std::process::exit(0);
}
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
pub fn enable_debug() -> Result<(), log::SetLoggerError> {

    // Remove all log files in the log directory but not the directory it self.
    let target_dir = path::get_logs_path();
    match fs::remove_dir_all(&target_dir) {
        Ok(_) => {
            log::debug!("Log files removed.");
        }
        Err(e) => {
            log::debug!("Log files not removed.");
            log::debug!("Error: {}", e);
        }
    }

    if std::env::var("RUST_LOG") == Ok("debug".to_string()) {
        log::debug!("Debug mode is already enabled.");
        Ok(())
    } else {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RUST_BACKTRACE", "1");
        
        // 파일 로거를 생성한다.
        // 패턴: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        let logfile_trace = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}:{m}{n}")))
        .build(target_dir.join("trace.log"))
        .unwrap();

        let logfile_debug = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}:{m}{n}")))
        .build(target_dir.join("debug.log"))
        .unwrap();

        let logfile_info = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}:{m}{n}")))
        .build(target_dir.join("info.log"))
        .unwrap();

        let logfile_warn = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}:{m}{n}")))
        .build(target_dir.join("warn.log"))
        .unwrap();

        let logfile_error = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {l}:{m}{n}")))
        .build(target_dir.join("error.log"))
        .unwrap();


        let config: Config;
        let result = Config::builder()
            // Debug 메세지를 StdOut으로 출력하는 로거를 생성한다.
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Debug)))
                    .build("stdout", Box::new(ConsoleAppender::builder().target(Target::Stdout).build())),
            )

            // Trace 메세지를 log/trace.log 파일로 출력하는 로거를 생성한다.
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Trace)))
                    .build("trace", Box::new(logfile_trace))
            )

            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Debug)))
                    .build("debug", Box::new(logfile_debug))
            )
            
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                    .build("info", Box::new(logfile_info))
            )

            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Warn)))
                    .build("warn", Box::new(logfile_warn))
            )

            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Error)))
                    .build("error", Box::new(logfile_error))
            )
            
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("trace")
                    .appender("debug")
                    .appender("info")
                    .appender("warn")
                    .appender("error")
                    .build(log::LevelFilter::Debug),
            );
        match result {
            Ok(conf) => {
                config = conf;
            }
            Err(e) => {
                panic!("Failed to build log4rs config: {}", e);
            }
        }

        let _handle;
        match log4rs::init_config(config) {
            Ok(handle) => {
                _handle = handle;
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to initialize log4rs: {}", e);
                Err(e)
            }
        }
    }
}