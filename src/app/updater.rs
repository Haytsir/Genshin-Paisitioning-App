use log::debug;
use serde_json::Value;
use crate::app::terminate_process;
use crate::models::{AppConfig, RequestDataTypes, RequestEvent, SendEvent, WsEvent};
use crate::models::UpdateInfo;
use crate::views::confirm::confirm_dialog;
use crate::app::path;
use crate::websocket::WebSocketHandler;
use std::collections::HashMap;
use std::fs::{File, self};
use std::path::PathBuf;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::events::EventBus;
use std::error::Error;
use std::sync::Arc;
use futures::StreamExt;
use reqwest::Client as StreamClient;
use std::cmp::min;

#[derive(Debug, Serialize, Deserialize)]
struct GithubCache {
    timestamp: u64,
    data: Value,
}

impl GithubCache {
    fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // 2시간 이내의 캐시만 유효
        now - self.timestamp < 7200
    }
}

fn get_cache_file_path(owner: &str, repo: &str) -> PathBuf {
    path::get_cache_path().join(format!("github_{}_{}.cache", owner, repo))
}

fn save_to_cache(owner: &str, repo: &str, data: &Value) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    let cache = GithubCache {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        data: data.clone(),
    };
    
    let cache_path = get_cache_file_path(owner, repo);
    let cache_dir = cache_path.parent().unwrap();
    std::fs::create_dir_all(cache_dir)?;
    
    let cache_str = serde_json::to_string(&cache)?;
    std::fs::write(cache_path, cache_str)?;
    Ok(())
}

fn load_from_cache(owner: &str, repo: &str) -> std::result::Result<Option<Value>, Box<dyn Error + Send + Sync>> {
    let cache_path = get_cache_file_path(owner, repo);
    if !cache_path.exists() {
        return Ok(None);
    }

    let cache_str = std::fs::read_to_string(cache_path)?;
    let cache: GithubCache = serde_json::from_str(&cache_str)?;
    
    if cache.is_valid() {
        Ok(Some(cache.data))
    } else {
        Ok(None)
    }
}

async fn fetch_app_version_on_github(owner: &str, repo: &str, force: bool) -> Result<Value, Box<dyn Error + Send + Sync>> {
    debug!("fetch_app_version_on_github");
    
    // 캐시 확인
    if !force {
        if let Some(cached_data) = load_from_cache(owner, repo)? {
            debug!("Using cached GitHub API response");
            return Ok(cached_data);
        }
    }

    // 캐시가 없거나 만료된 경우 GitHub API 호출
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let response = client.get(&url)
        .header("User-Agent", "reqwest")
        .header("Accept", "application/vnd.github.v3+json")
        .header("Content-Type", "application/json")
        .send()
        .await?;
        
    log::debug!("{:#?}", &response.status());
    if response.status().as_u16() != 200 {
        let e = format!("Error: Github API 요청에 실패했습니다: {}", &response.text().await?);
        return Err(e.into());
    }
    
    let json: Value = serde_json::from_str(&response.text().await?)?;
    
    // 응답 캐시에 저장
    save_to_cache(owner, repo, &json)?;
    
    Ok(json)
}

pub async fn download_app(
    ws_handler: WebSocketHandler, 
    requester_id: String, 
    force: bool
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    debug!("download_app");
    let json: Value = fetch_app_version_on_github("Haytsir", "Genshin-Paisitioning-App", force).await?;
    let cache_dir = path::get_cache_path();
    // 태그 이름 가져오
    let version = env!("CARGO_PKG_VERSION");
    let release_name = &json["tag_name"].as_str().unwrap_or("")[1..];
    let release_display_name = &json["name"].as_str().unwrap_or("");

    // 업데이트를 요청한 유저에게 보낼 update info 생성
    let mut update_info = UpdateInfo {
        target_type: "app".to_string(),
        current_version: version.to_string(),
        target_version: release_name.to_string(),
        display_version_name: release_display_name.to_string(),
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
        send_app_update_info(ws_handler.clone(), requester_id.clone(), Some(update_info)).await?;
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
        let ws_handler = ws_handler.clone();

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
                    ws_handler.clone(),
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
                    send_app_update_info(ws_handler.clone(), requester_id.clone(), Some(update_info)).await?;

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

pub async fn download_cvat(
    ws_handler: WebSocketHandler, 
    requester_id: String, 
    force: bool
) -> Result<(), Box<dyn Error + Send + Sync>> {
    debug!("download_cvat");
    let lib_path = path::get_lib_path();    
    let cache_dir = path::get_cache_path();

    let json: Value = fetch_app_version_on_github("Haytsir", "gpa-lib-mirror", force).await?;
    
    // 태그 이름 가져오기
    let version = get_local_version(&lib_path);
    let release_name = json["tag_name"].as_str().unwrap_or("");
    let release_display_name = json["name"].as_str().unwrap_or("");

    // 업데이트를 요한 유저에게 보낼 update info 생성
    let mut update_info = UpdateInfo {
        target_type: "cvat".to_string(),
        current_version: version.to_string(),
        target_version: release_name.to_string(),
        display_version_name: release_display_name.to_string(),
        downloaded: 0,
        file_size: 0,
        percent: 0.0,
        done: false,
        updated: true,
    };

    if lib_path.join("cvAutoTrack.dll").exists() {
        // 버전 비교, 최신 버전이 자가 더 낮은 경우가 있으니 파일 수정 시간으로 비교
        let last_file_modified = get_file_modified_time(&lib_path.join("cvAutoTrack.dll"))?;
        let last_lib_published = parse_iso8601(json["published_at"].as_str().unwrap_or(""))?;

        if last_file_modified > last_lib_published/* compare_versions(&version, json["tag_name"].as_str().unwrap_or("")) */ {
            log::debug!("CVAT가 최신 버전입니다. ({})", release_name);
            update_info.done = true;
            update_info.updated = false;
            // 처음 상황을 전송한다.
            send_lib_update_info(ws_handler.clone(), requester_id.clone(), Some(update_info)).await?;
            return Ok(());
        } else {
            log::debug!(
                "현재 CVAT 버전은 최신 버전 {}과 일치하지 않습니다.",
                release_name
            );
        }
    }

    crate::cvat::unload_cvat().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    // 첫 번째 첨부 파일 가져오기
    let assets = &json["assets"];
    for asset in assets.as_array().unwrap() {
        let asset_url = asset["browser_download_url"].as_str().unwrap();
        let asset_name = asset["name"].as_str().unwrap();
        log::debug!("{} 다운로드 시도", asset_url);
        log::debug!("파일명: {}", asset_name);
        let ws_handler = ws_handler.clone();
        let update_info = update_info.clone();

        // github에서 은 파일이 .zip 확장자인 경우
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
                    ws_handler.clone(),
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
            // 파일 다운로드  저장
            let runtime = tokio::runtime::Runtime::new().unwrap();
            // arch_path: .7z 파일의 경로
            let file_path = cache_dir.join(if asset_name.ends_with(".md5") {"cvAutoTrack.md5"} else {asset_name});
            debug!("file_path: {:?}", file_path);
            let target_path = lib_path.clone().join(if asset_name.ends_with(".md5") {"cvAutoTrack.md5"} else {asset_name}); // ? cvAutoTrack instead of asset_name
            let res = runtime.handle().block_on(download_file(
                asset_url,
                &file_path,
                ws_handler.clone(),
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
    send_lib_update_info(ws_handler.clone(), requester_id.clone(), Some(update_info)).await?;

    Ok(())
}

fn get_file_modified_time(file_path: &PathBuf) -> std::result::Result<std::time::SystemTime, Box<dyn Error + Send + Sync>> {
    let metadata = std::fs::metadata(file_path)?;
    let modified_time = metadata.modified()?;
    Ok(modified_time)
}
fn parse_iso8601(date: &str) -> std::result::Result<std::time::SystemTime, Box<dyn Error + Send + Sync>> {
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

use std::io::Write;

pub async fn download_file(
    url: &str,
    path: &PathBuf,
    ws_handler: WebSocketHandler,
    mut update_info: UpdateInfo,
    requester_id: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            ws_handler.send_to(requester_id.clone(), SendEvent::from(WsEvent::UpdateInfo { 
                info: Some(update_info.clone())
            })).await?;
        }
    }

    Ok(())
}

fn extract_files_from_zip(arch_path: &PathBuf, mappings: HashMap<&str, &PathBuf>) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
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

pub async fn register_events(
    _event_bus: &Arc<EventBus>, 
    ws_handler: &Arc<WebSocketHandler>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_handler_app = ws_handler.clone();
    let config = super::config::ConfigManager::global().get().await;
    let config_clone = config.clone();
    ws_handler.register("checkAppUpdate", move |id, params: RequestEvent| {
        let ws_handler = ws_handler_app.clone();
        let config = config_clone.clone();
        async move {
            let force = if let Some(data) = &params.data {
                match data {
                    RequestDataTypes::CheckAppUpdate(data) => data.force,
                    _ => false
                }
            } else {
                false
            };
            check_app_update(&config, id, (*ws_handler).clone(), force).await
        }
    }).await?;
    let ws_handler_lib = ws_handler.clone();
    let config_clone = config.clone();
    ws_handler.register("checkLibUpdate", move |id, params: RequestEvent| {
        let ws_handler = ws_handler_lib.clone();
        let config = config_clone.clone();
        async move {
            let force = if let Some(data) = &params.data {
                match data {
                    RequestDataTypes::CheckLibUpdate(data) => data.force,
                    _ => false
                }
            } else {
                false
            };
            check_lib_update(&config, id, (*ws_handler).clone(), force).await
        }
    }).await?;

    Ok(())
}

pub async fn check_app_update(
    config: &AppConfig, 
    client_id: String, 
    ws_handler: WebSocketHandler, 
    force: bool
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if cfg!(debug_assertions) {
        log::debug!("!!! 디버그 모드입니다 !!!");
        log::debug!("현재 버전을 계속 사용합니다!");
        send_app_update_info(ws_handler.clone(), client_id.clone(), None).await?;
        return Ok(());
    }

    if config.auto_app_update {
        match download_app(ws_handler.clone(), client_id.clone(), force).await {
            Ok(_) => {
                log::debug!("App Ready!");
                Ok(())
            }
            Err(e) => {
                log::error!("{}", e);
                log::debug!("현재 버전을 계속 사용합니다!");
                send_app_update_info(ws_handler.clone(), client_id.clone(), None).await?;
                Err(e)
            }
        }
    } else {
        log::debug!("자동 업데이트가 꺼져있습니다.");
        log::debug!("현재 버전을 계속 사용합니다!");
        send_app_update_info(ws_handler.clone(), client_id.clone(), None).await?;
        Ok(())
    }
}

pub async fn check_lib_update(
    config: &AppConfig, 
    client_id: String, 
    ws_handler: WebSocketHandler, 
    force: bool
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if config.auto_app_update {
        match download_cvat(ws_handler.clone(), client_id.clone(), force).await {
            Ok(_) => {
                log::debug!("Lib Ready!");
                Ok(())
            }
            Err(e) => {
                log::error!("{}", e);
                log::debug!("현재 버전을 계속 사용합니다!");
                send_lib_update_info(ws_handler.clone(), client_id.clone(), None).await?;
                Err(e)
            }
        }
    } else {
        log::debug!("자동 업데이트가 꺼져있습니다.");
        log::debug!("현재 버전을 계속 사용합니다!");
        send_lib_update_info(ws_handler.clone(), client_id.clone(), None).await?;
        Ok(())
    }
}


pub async fn send_app_update_info(
    ws_handler: WebSocketHandler,
    requester_id: String,
    update_info: Option<UpdateInfo>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let info = update_info.unwrap_or(UpdateInfo {
        target_type: "app".to_string(),
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        target_version: String::from(""),
        display_version_name: String::from(""),
        downloaded: 0,
        file_size: 0,
        percent: 0.0,
        done: true,
        updated: false
    });
    
    ws_handler.send_to(requester_id.clone(), SendEvent::from(WsEvent::UpdateInfo { 
        info: Some(info)
    })).await?;
    Ok(())
}

pub async fn send_lib_update_info(
    ws_handler: WebSocketHandler,
    requester_id: String,
    update_info: Option<UpdateInfo>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let info: UpdateInfo;
    let lib_path: PathBuf;
    let version_string: String;
    if update_info.is_none() {
        lib_path = path::get_lib_path();
        version_string = get_local_version(&lib_path);

        info = update_info.unwrap_or(UpdateInfo {
            target_type: "cvat".to_string(),
            current_version: version_string,
            target_version: String::from(""),
            display_version_name: String::from(""),
            downloaded: 0,
            file_size: 0,
            percent: 0.0,
            done: true,
            updated: false
        });
    } else {
        info = update_info.unwrap();
    }
    
    ws_handler.send_to(requester_id.clone(), SendEvent::from(WsEvent::UpdateInfo { 
        info: Some(info)
    })).await?;
    Ok(())
}