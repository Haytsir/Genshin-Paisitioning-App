use crossbeam_channel::Sender;
use log::debug;
use reqwest::blocking::Client;
use serde_json::Value;
use crate::app::terminate_process;
//use sevenz_rust::default_entry_extract_fn;
use crate::models::{WsEvent, AppConfig};
use crate::models::{AppEvent, UpdateInfo};
use crate::views::confirm::confirm_dialog;
use crate::app::path;
use std::collections::HashMap;
use std::fs::{File, self};
use std::path::PathBuf;
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn fetch_app_version_on_github(owner: &str, repo: &str) -> Result<Value> {
    debug!("fetch_app_version_on_github");
    // 최신 릴리스 정보 가져오기
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let response = client.get(&url)
                                    .header("User-Agent", "reqwest")
                                    .header("Accept", "application/vnd.github.v3+json")
                                    .header("Content-Type", "application/json")
                                    .send()?;
    log::debug!("{:#?}", &response.status());
    if response.status().as_u16() > 400 {
        let e = format!("Error: Github API 요청에 실패했습니다: {}", &response.text()?);
        return Err(e.into());
    }
    let json: serde_json::Result<Value> = serde_json::from_str(&response.text()?);
    match json {
        Ok(json) => {
            let version = json["tag_name"].as_str().unwrap();
            Ok(json)
        }
        Err(e) => {
            log::error!("{}", e);
            Err(e.into())
        }
    }
}
pub fn download_app(sender: Option<Sender<WsEvent>>, requester_id: String) -> Result<()> {
    debug!("download_app");
    let json: Value = fetch_app_version_on_github("Haytsir", "Genshin-Paisitioning-App")?;
    let cache_dir = path::get_cache_path();
    // 태그 이름 가져오기
    let version = env!("CARGO_PKG_VERSION");
    let release_name = &json["name"].as_str().unwrap_or("")[1..];

    // 업데이트를 요청한 유저에게 보낼 update info 생성
    let mut update_info = UpdateInfo {
        target_type: "app".to_string(),
        current_version: version.to_string(),
        target_version: release_name.to_string(),
        downloaded: 0,
        file_size: 0,
        percent: 0.0,
        done: false,
        updated: true
    };

    // 버전 비교
    if compare_versions(version, &json["tag_name"].as_str().unwrap_or("")) {
        log::debug!("GPA가 최신 버전입니다. ({})", release_name);
        update_info.done = true;
        update_info.updated = false;
        // 처음 상황을 전송한다.
        let _ = sender.unwrap().send(WsEvent::UpdateInfo(
            update_info,
            requester_id,
        ));
        return Ok(());
    } else {
        log::debug!(
            "현재 GPA 버전은 최신 버전 {}과 일치하지 않습니다.",
            release_name
        );
    }

    // 첨부 파일 가져오기
    let assets = &json["assets"];
    for asset in assets.as_array().unwrap() {
        let asset_url = asset["browser_download_url"].as_str().unwrap();
        let asset_name = asset["name"].as_str().unwrap();
        log::debug!("{} 다운로드 시도", asset_url);
        log::debug!("파일명: {}", asset_name);
        let sender = sender.clone();

        // github에서 받은 파일이 .zip 확장자인 경우
        if asset_name.ends_with(".zip") {
            // 파일 다운로드 및 저장
            let runtime = tokio::runtime::Runtime::new().unwrap();
            // arch_path: .zip 파일의 경로
            let arch_path = cache_dir.join(asset_name);
            let res = runtime
                .handle()
                .block_on(download_file(
                    asset_url,
                    &arch_path,
                    sender.clone(),
                    update_info.clone(),
                    requester_id.clone(),
                ))
                .and_then(|()| {
                    std::thread::sleep(Duration::from_millis(1000));

                    // 추출할 파일 확장자와, 대상 경로를 가진 해쉬맵 구성
                    let mut mappings = HashMap::new();
                    log::debug!("{}", arch_path.display());

                    mappings.insert("exe", &cache_dir);
                    extract_files_from_zip(&arch_path, mappings)?;
                    return Ok(());
                });
            match res {
                Ok(_) => {
                    let remove_res = std::fs::remove_file(cache_dir.join(asset_name));
                    match remove_res {
                        Ok(_) => {}
                        Err(e) => {
                            log::debug!("{}", e);
                        }
                    }

                    let current_exe = std::env::current_exe().unwrap();
                    let exe_name = current_exe.file_name().unwrap();

                    log::debug!("Updating...");
                    self_replace::self_replace(&cache_dir.join(exe_name))?;
                    fs::remove_file(&cache_dir.join(exe_name))?;
                    let _ = confirm_dialog(env!("CARGO_PKG_DESCRIPTION"), "GPA 업데이트를 완료했습니다.", false);

                    let mut update_info = update_info.clone();
                    update_info.done = true;
                    let _ = sender.as_ref().unwrap().send(WsEvent::UpdateInfo(
                        update_info,
                        requester_id.clone(),
                    ));

                    std::thread::sleep(Duration::from_millis(1000));
                    terminate_process();
                }
                Err(e) => {
                    log::debug!("{}", e)
                }
            }
        }
    }
    return Ok(());
}

pub fn download_cvat(sender: Option<Sender<WsEvent>>, requester_id: String) -> Result<()> {
    debug!("download_cvat");
    let lib_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("cvAutoTrack"); // 저장할 파일 경로 및 이름
    
    let cache_dir = path::get_cache_path();

    // 최신 릴리스 정보 가져오기
    let json: Value = fetch_app_version_on_github("Haytsir", "gpa-lib-mirror")?;

    debug!("json: {:#?}", json);
    // 태그 이름 가져오기
    let version = get_local_version(&lib_path);
    let release_name = json["name"].as_str().unwrap_or("");

    // 업데이트를 요청한 유저에게 보낼 update info 생성
    let mut update_info = UpdateInfo {
        target_type: "cvat".to_string(),
        current_version: version.to_string(),
        target_version: release_name.to_string(),
        downloaded: 0,
        file_size: 0,
        percent: 0.0,
        done: false,
        updated: true,
    };

    if lib_path.join("cvAutoTrack.dll").exists() {
        // 버전 비교, 최신 버전이 버전 숫자가 더 낮은 경우가 있으니 파일 수정 시간으로 비교
        let last_file_modified = get_file_modified_time(&lib_path.join("cvAutoTrack.dll"))?;
        let last_lib_published = parse_iso8601(json["published_at"].as_str().unwrap_or(""))?;

        if last_file_modified > last_lib_published/* compare_versions(&version, json["tag_name"].as_str().unwrap_or("")) */ {
            log::debug!("CVAT가 최신 버전입니다. ({})", release_name);
            update_info.done = true;
            update_info.updated = false;
            // 처음 상황을 전송한다.
            let _ = sender.unwrap().send(WsEvent::UpdateInfo(
                update_info,
                requester_id,
            ));
            return Ok(());
        } else {
            log::debug!(
                "현재 CVAT 버전은 최신 버전 {}과 일치하지 않습니다.",
                release_name
            );
        }
    }

    // 첫 번째 첨부 파일 가져오기
    let assets = &json["assets"];
    for asset in assets.as_array().unwrap() {
        let asset_url = asset["browser_download_url"].as_str().unwrap();
        let asset_name = asset["name"].as_str().unwrap();
        log::debug!("{} 다운로드 시도", asset_url);
        log::debug!("파일명: {}", asset_name);
        let sender = sender.clone();
        let update_info = update_info.clone();

        // github에서 받은 파일이 .zip 확장자인 경우
        if asset_name.ends_with(".zip") {
            // 파일 다운로드 및 저장
            let runtime = tokio::runtime::Runtime::new().unwrap();
            // arch_path: .zip 파일의 경로
            let arch_path = cache_dir.join(asset_name);
            let target_path = lib_path.clone();
            let res = runtime
                .handle()
                .block_on(download_file(
                    asset_url,
                    &arch_path,
                    sender.clone(),
                    update_info,
                    requester_id.clone(),
                ))
                .and_then(|()| {
                    std::thread::sleep(Duration::from_millis(1000));

                    // 추출할 파일 확장자와, 대상 경로를 가진 해쉬맵 구성
                    let mut mappings = HashMap::new();
                    log::debug!("{}", arch_path.display());

                    mappings.insert("dll", &target_path);
                    //                mappings.insert("md5", &lib_path);
                    //                mappings.insert("tag", &lib_path);
                    std::fs::create_dir_all(lib_path.clone())?;
                    extract_files_from_zip(&arch_path, mappings)?;
                    Ok(())
                });
            match res {
                Ok(_) => {
                    let remove_res = std::fs::remove_file(cache_dir.join(asset_name));
                    match remove_res {
                        Ok(_) => {}
                        Err(e) => {
                            log::debug!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    log::debug!("{}", e)
                }
            }
        } else if asset_name.ends_with(".md5") || asset_name.ends_with(".tag") {
            // 파일 다운로드 및 저장
            let runtime = tokio::runtime::Runtime::new().unwrap();
            // arch_path: .7z 파일의 경로
            let file_path = cache_dir.join(if asset_name.ends_with(".md5") {"cvAutoTrack.md5"} else {asset_name});
            debug!("file_path: {:?}", file_path);
            let target_path = lib_path.clone().join(if asset_name.ends_with(".md5") {"cvAutoTrack.md5"} else {asset_name}); // ? cvAutoTrack instead of asset_name
            let res = runtime.handle().block_on(download_file(
                asset_url,
                &file_path,
                sender.clone(),
                update_info,
                requester_id.clone(),
            ));

            match res {
                Ok(_) => {
                    std::fs::create_dir_all(lib_path.clone())?;
                    std::fs::rename(file_path, target_path)?;
                }
                Err(e) => {
                    log::debug!("{}", e)
                }
            }
        }
    }

    update_info.done = true;
    let _ = sender.as_ref().unwrap().send(WsEvent::UpdateInfo(
        update_info,
        requester_id,
    ));

    Ok(())
}
fn get_file_modified_time(file_path: &PathBuf) -> Result<std::time::SystemTime> {
    let metadata = std::fs::metadata(file_path)?;
    let modified_time = metadata.modified()?;
    Ok(modified_time)
}
fn parse_iso8601(date: &str) -> Result<std::time::SystemTime> {
    let datetime = chrono::DateTime::parse_from_rfc3339(date)?;
    Ok(datetime.into())
}
fn get_local_version(lib_path: &PathBuf) -> String {
    // TODO:
    log::debug!("{}", lib_path.to_str().unwrap());
    match std::fs::read_to_string(lib_path.join("version.tag")) {
        Ok(contents) => contents.trim().to_string(),
        Err(_) => {
            log::debug!("Error: Failed to read version tag file.");
            String::from("")
        }
    }
}

pub fn compare_versions(version: &str, release_name: &str) -> bool {
    debug!("compare_versions({}, {})", version, release_name);
    let version = version.trim_start_matches('v');
    let release_name = release_name.trim_start_matches('v');
    let latest_version = release_name.trim();
    if version.eq(latest_version) {
        return true;
    } else {
        if version.len() > 0 && release_name.len() > 0 {
            let current_semver = version.split('.').map(|s| s.parse::<i32>().unwrap()).collect::<Vec<i32>>();
            let latest_semver = latest_version.split('.').map(|s| s.parse::<i32>().unwrap()).collect::<Vec<i32>>();
            if current_semver[0] > latest_semver[0] {
                return true;
            } else if current_semver[0] == latest_semver[0] {
                if current_semver[1] > latest_semver[1] {
                    return true;
                } else if current_semver[1] == latest_semver[1] {
                    if current_semver[2] >= latest_semver[2] {
                        return true;
                    }
                }
            }
        }
        return false;
    }
        
}

use futures_util::StreamExt;
use reqwest::Client as StreamClient;
use std::cmp::min;
use std::io::Write;

pub async fn download_file(
    url: &str,
    path: &PathBuf,
    sender: Option<Sender<WsEvent>>,
    mut update_info: UpdateInfo,
    requester_id: String,
) -> Result<()> {
    // Reqwest setup
    let client = StreamClient::new();
    let res = client
        .get(url)
        .header("User-Agent", "reqwest")
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;
    update_info.file_size = total_size;

    // download chunks
    let mut file = File::create(path).or(Err(format!(
        "Failed to create file '{}'",
        path.clone().to_str().unwrap()
    )))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    let mut old_percent: f64 = -1.0;
    let sender = sender.as_ref().unwrap();
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file".to_string()))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file".to_string()))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let mut update_info = update_info.clone();
        let requester_id = requester_id.clone();
        update_info.downloaded = downloaded;
        let percent = f64::trunc((downloaded as f64 * 100.0 / total_size as f64) * 10.0) / 10.0;
        if percent - old_percent > 1.0 {
            old_percent = percent;
            update_info.percent = percent;
            let _ = sender.send(WsEvent::UpdateInfo(update_info, requester_id));
        }
    }

    Ok(())
}

fn extract_files_from_zip(arch_path: &PathBuf, mappings: HashMap<&str, &PathBuf>) -> Result<()> {
    let file = fs::File::open(arch_path).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let comment = file.comment();
        if !comment.is_empty() {
            println!("File {i} comment: {comment}");
        }

        if let Some(ext) = file.enclosed_name().unwrap()
            .extension()
            .and_then(|e| e.to_str())
        {
            if let Some(out_path) = mappings.get(ext) {
                log::debug!("압축 해제 대상 경로: {:?}", out_path.to_str().unwrap());
                let mut out_file_path = PathBuf::from(out_path.to_str().unwrap());
                out_file_path = out_file_path.join(file.enclosed_name().unwrap());

                // create parent directories if necessary
                if let Some(parent) = out_file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&out_file_path).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
    Ok(())
}

pub fn check_app_update(config: config::Config, client_id: String, tx: Option<Sender<WsEvent>>) -> Result<()> {
    debug!("check_app_update");
    let app_config: AppConfig = config.clone().try_deserialize().map_err(|e| {
        log::error!("{}", e);
        Box::<dyn std::error::Error + Send + Sync>::from(e)
    })?;
    log::debug!("app_config: {:?}", app_config);
    if app_config.auto_app_update {
        let result = download_app(tx.clone(), client_id.clone());
        match result {
            Ok(_) => {
                log::debug!("App Ready!");
                return result;
            }
            Err(e) => {
                log::error!("{}", e);
                log::debug!("현재 버전을 계속 사용합니다!");
                let tx_result = send_app_update_info(tx.clone(), client_id.clone(), None);
                match tx_result {
                    Ok(_) => {
                        return Err(e);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        return Err(e);
                    }
                }
            }
        }
    } else {
        log::debug!("자동 업데이트가 꺼져있습니다.");
        log::debug!("현재 버전을 계속 사용합니다!");
        let result = send_app_update_info(tx.clone(), client_id.clone(), None);
        match result {
            Ok(_) => {
                return result;
            }
            Err(e) => {
                log::error!("{}", e);
                return Err(e);
            }   
        }
    }
}

pub fn check_lib_update(config: config::Config, client_id: String, tx: Option<Sender<WsEvent>>) -> Result<()> {
    //let app_config: std::result::Result<AppConfig, config::ConfigError>  = config.clone().try_deserialize();
    let app_config: AppConfig = config.clone().try_deserialize().map_err(|e| {
        log::error!("{}", e);
        Box::<dyn std::error::Error + Send + Sync>::from(e)
    })?;
    log::debug!("app_config: {:?}", app_config);
    if app_config.auto_app_update {
        let result = super::updater::download_cvat(tx.clone(), client_id.clone());
        match result {
            Ok(_) => {
                log::debug!("Lib Ready!");
                return result;
            }
            Err(e) => {
                log::error!("{}", e);
                log::debug!("현재 버전을 계속 사용합니다!");
                let result = send_lib_update_info(tx.clone(), client_id.clone(), None);
                match result {
                    Ok(_) => {
                        return Err(e);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        return Err(e);
                    }
                }
            }
        }
    } else {
        log::debug!("자동 업데이트가 꺼져있습니다.");
        log::debug!("현재 버전을 계속 사용합니다!");
        let result = send_lib_update_info(tx.clone(), client_id.clone(), None);
        match result {
            Ok(_) => {
                return result;
            }
            Err(e) => {
                log::error!("{}", e);
                return Err(e);
            }
        }
    }
}

fn send_app_update_info(sender: Option<Sender<WsEvent>>, requester_id: String, update_info: Option<UpdateInfo>) -> Result<()> {
    // 업데이트를 요청한 유저에게 보낼 update info 생성
    let info = update_info.unwrap_or(
    UpdateInfo {
        target_type: "app".to_string(),
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        target_version: String::from(""),
        downloaded: 0,
        file_size: 0,
        percent: 0.0,
        done: true,
        updated: false
    });
    match sender.as_ref().unwrap().send(WsEvent::UpdateInfo(
        info,
        requester_id,
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error: {}", e);
            Err(e.try_into().unwrap())
        }
    }
}

fn send_lib_update_info(sender: Option<Sender<WsEvent>>, requester_id: String, update_info: Option<UpdateInfo>) -> Result<()> {
    let info: UpdateInfo;
    let lib_path: PathBuf;
    let version_string: String;
    if update_info.is_none() {
        lib_path = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("cvAutoTrack"); // 저장할 파일 경로 및 이름
        version_string = get_local_version(&lib_path);

        info = update_info.unwrap_or(UpdateInfo {
            target_type: "cvat".to_string(),
            current_version: version_string,
            target_version: String::from(""),
            downloaded: 0,
            file_size: 0,
            percent: 0.0,
            done: true,
            updated: false
        });
    } else {
        info = update_info.unwrap();
    }
    
    
    match sender.as_ref().unwrap().send(WsEvent::UpdateInfo(
        info,
        requester_id,
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error: {}", e);
            Err(e.try_into().unwrap())
        }
    }
}