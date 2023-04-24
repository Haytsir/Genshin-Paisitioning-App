use directories::ProjectDirs;
use std::ffi::CString;
use std::path::Path;
use std::process::Command;
use std::ptr::null_mut;
use sysinfo::{ProcessExt, System, SystemExt};
use winapi::um::shellapi::ShellExecuteA;
use winapi::um::{
    processthreadsapi::{GetCurrentProcess, OpenProcessToken},
    securitybaseapi::GetTokenInformation,
    winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY},
};

// 현재 프로그램이 프로젝트 디렉토리에서 실행중인지 확인한다.
pub fn check_proj_directory() -> bool {
    // 프로젝트 디렉토리 정의
    if let Some(proj_dirs) = ProjectDirs::from("com", "genshin-paisitioning", "") {
        // target_dir의 내용이 프로젝트 디렉토리의 Root가 된다.
        let target_dir = proj_dirs.cache_dir().parent().unwrap();
        log::debug!("Project Directory: {}", target_dir.display());
        // 먼저 프로젝트 디렉토리가 존재하지 않는다면 생성한다.
        match std::fs::create_dir_all(proj_dirs.cache_dir()) {
            Ok(_) => {
                log::debug!("Project Directory: 생성 성공");
            }
            Err(e) => {
                log::debug!("Project Directory: 생성 실패");
                log::debug!("Error: {}", e);
            }
        }

        // Current Working Directory를 얻어낸다.
        let current_exe = std::env::current_exe().unwrap();
        let cwd = current_exe.parent().unwrap();
        let exe_name = current_exe.file_name().unwrap();

        // 현재 작업 디렉토리와 프로젝트 디렉토리를 대조하고,
        // 일치하지 않으면 파일을 복사한다.
        if !cwd.eq(target_dir) {
            match std::fs::copy(&current_exe, target_dir.join(exe_name)) {
                Ok(_) => {
                    log::debug!("실행 파일을 Project Directory로 복사 성공");
                }
                Err(e) => {
                    log::debug!("실행 파일을 Project Directory로 복사 실패");
                    log::debug!("Error: {}", e);
                }
            }
            return false;
        }
    }
    true
}

pub fn check_elevation(target: &Path, args: Vec<&str>) -> bool {
    unsafe {
        if is_elevated() {
            return true;
        } else {
            ShellExecuteA(
                null_mut(),
                CString::new("runas").unwrap().as_ptr(),
                CString::new(target.to_str().unwrap()).unwrap().as_ptr(),
                CString::new(args.join(" ")).unwrap().as_ptr(),
                null_mut(),
                1,
            );
        }
    }
    false
}

pub fn run_shell_execute(target: &Path, args: Vec<String>) {
    unsafe {
        ShellExecuteA(
            null_mut(),
            null_mut(),
            CString::new(target.to_str().unwrap()).unwrap().as_ptr(),
            CString::new(args.join(" ")).unwrap().as_ptr(),
            null_mut(),
            1,
        );
    }
}

pub fn is_elevated() -> bool {
    let mut h_token: HANDLE = null_mut();
    let mut token_ele: TOKEN_ELEVATION = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut size: u32 = 0u32;
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut h_token);
        GetTokenInformation(
            h_token,
            TokenElevation,
            &mut token_ele as *const _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        token_ele.TokenIsElevated == 1
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
    let mut system = System::new_all();
    system.refresh_all();
    let mut count = 0;
    for process in system.processes_by_exact_name("genshin_paisitioning_app.exe") {
        if process.pid() != <sysinfo::Pid as sysinfo::PidExt>::from_u32(std::process::id()) {
            count += 1;
        }
    }
    if count >= 1 {
        return true;
    }
    false
}

pub fn terminate_process() {
    std::process::exit(0);
}
use std::sync::{Once};
static ONCE_DEBUG: Once = Once::new();
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
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    let proj_dirs = ProjectDirs::from("com", "genshin-paisitioning", "").unwrap();
    // target_dir의 내용이 프로젝트 디렉토리의 Root가 된다.
    let target_dir = proj_dirs.cache_dir().parent().unwrap().join("logs");
    
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


    let config = Config::builder()
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
        )
        .unwrap();

    let _handle = log4rs::init_config(config)?;

    Ok(())
}