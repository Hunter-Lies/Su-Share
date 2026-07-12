use std::path::PathBuf;
use std::sync::Arc;
use rand::Rng;
use tauri_plugin_dialog::DialogExt;
use crate::state::{AppState, SharedFile, BundleFileInfo, ServerInfo, FileEntry, ReceivedFile, save_shares};
use crate::qr::encode_qr_png;

#[tauri::command]
pub fn generate_qr(text: String, size: Option<u32>) -> Result<String, String> {
    encode_qr_png(&text, size.unwrap_or(256))
}

#[tauri::command]
pub fn get_send_qr(state: tauri::State<Arc<AppState>>, size: Option<u32>) -> Result<String, String> {
    encode_qr_png(&format!("http://{}:{}", state.lan_ip, state.port), size.unwrap_or(256))
}

#[tauri::command]
pub async fn pick_files(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    match app.dialog().file().blocking_pick_files() {
        Some(paths) => Ok(paths.iter().map(|p| p.to_string()).collect()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub fn share_files(paths: Vec<String>, state: tauri::State<Arc<AppState>>) -> Result<SharedFile, String> {
    println!("[Su!] share_files called with {} path(s)", paths.len());
    let mut map = state.files.lock().map_err(|e| e.to_string())?;
    let mut bundles = state.bundles.lock().map_err(|e| e.to_string())?;
    let bundle_id: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
    let bundle_id = bundle_id.to_lowercase();
    let mut file_ids = Vec::new();
    let mut bundle_files = Vec::new();
    let display_name: String;
    let mut total_size: u64 = 0;
    for p in paths {
        let pb = PathBuf::from(&p);
        if !pb.exists() { continue; }
        let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
        let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        let fid: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
        let fid = fid.to_lowercase();
        total_size += size;
        map.insert(fid.clone(), FileEntry { name: name.clone(), path: p, size });
        file_ids.push(fid);
        bundle_files.push(BundleFileInfo { name, size });
    }
    if bundle_files.len() == 1 {
        display_name = bundle_files[0].name.clone();
    } else {
        display_name = format!("{} files", bundle_files.len());
    }
    let url = format!("http://{}:{}/s/{}", state.lan_ip, state.port, bundle_id);
    bundles.insert(bundle_id.clone(), file_ids);
    drop(map);
    drop(bundles);
    let result = SharedFile { id: bundle_id, name: display_name, size: total_size, url: url.clone(), files: bundle_files };
    save_shares(&state);
    Ok(result)
}

#[tauri::command]
pub fn get_server_info(state: tauri::State<Arc<AppState>>) -> ServerInfo {
    let base = format!("http://{}:{}", state.lan_ip, state.port);
    ServerInfo { url: base.clone(), lan_ip: state.lan_ip.clone(), port: state.port, send_url: base }
}

#[tauri::command]
pub fn stop_share(id: String, state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    // id is the bundle_id - remove bundle and all its files
    if let Ok(mut bundles) = state.bundles.lock() {
        if let Some(file_ids) = bundles.remove(&id) {
            if let Ok(mut files) = state.files.lock() {
                for fid in &file_ids {
                    files.remove(fid);
                }
            }
        }
        }
    save_shares(&state);
    Ok(())
}

#[tauri::command]
pub fn get_active_shares(state: tauri::State<Arc<AppState>>) -> Vec<SharedFile> {
    let bundles = state.bundles.lock().unwrap();
    let files = state.files.lock().unwrap();
    let mut result = Vec::new();
    for (bid, fids) in bundles.iter() {
        let url = format!("http://{}:{}/s/{}", state.lan_ip, state.port, bid);
        let mut bundle_files = Vec::new();
        let mut total_size: u64 = 0;
        for fid in fids {
            if let Some(entry) = files.get(fid) {
                total_size += entry.size;
                bundle_files.push(BundleFileInfo { name: entry.name.clone(), size: entry.size });
            }
        }
        let display_name = if bundle_files.len() == 1 {
            bundle_files[0].name.clone()
        } else {
            format!("{} files", bundle_files.len())
        };
        result.push(SharedFile {
            id: bid.clone(),
            name: display_name,
            size: total_size,
            url,
            files: bundle_files,
        });
    }
    result
}
#[tauri::command]
pub fn get_received_files(state: tauri::State<Arc<AppState>>) -> Vec<ReceivedFile> {
    state.received.lock().map(|r| {
        r.iter().map(|item| {
            let mut item = item.clone();
            item.exists = std::path::Path::new(&item.path).exists();
            item
        }).collect()
    }).unwrap_or_default()
}

#[tauri::command]
pub fn clear_all_shares(state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    {
        state.files.lock().map_err(|e| e.to_string())?.clear();
        state.bundles.lock().map_err(|e| e.to_string())?.clear();
    }
    save_shares(&state);
    Ok(())
}

#[tauri::command]
pub fn clear_received(state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    state.received.lock().map_err(|e| e.to_string())?.clear();
    let path = state.data_dir.join("received.json");
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    match app.dialog().file().blocking_pick_folder() {
        Some(path) => Ok(vec![path.to_string()]),
        None => Ok(vec![]),
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn create_shortcut(_state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_path = exe.to_string_lossy().to_string();
    let desktop = dirs::desktop_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    let lnk_path = desktop.join("Su!.lnk");
    let work_dir = exe.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let ps = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{lnk}'); $s.TargetPath = '{exe}'; $s.WorkingDirectory = '{wd}'; $s.Save()",
        lnk = lnk_path.to_string_lossy().replace("'", "''"),
        exe = exe_path.replace("'", "''"),
        wd = work_dir.replace("'", "''")
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output()
        .map_err(|e| format!("PowerShell failed: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed: {}", err))
    }
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .args(["/select,", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn resize_window(window: tauri::Window, width: f64, height: f64) {
    use tauri::LogicalSize;
    window.set_size(LogicalSize::new(width, height)).ok();
    window.center().ok();
}

#[tauri::command]
pub fn minimize_window(window: tauri::Window) { window.minimize().ok(); }

#[tauri::command]
pub fn toggle_maximize(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) { window.unmaximize().ok(); }
    else { window.maximize().ok(); }
}

#[tauri::command]
pub fn set_tray_mode(state: tauri::State<Arc<AppState>>, enabled: bool) {
    *state.tray_mode.lock().unwrap() = enabled;
}

#[tauri::command]
pub fn get_cli_paths(state: tauri::State<Arc<AppState>>) -> Vec<String> {
    let mut pending = state.pending_cli_paths.lock().unwrap();
    let paths = pending.clone();
    pending.clear();
    paths
}



#[tauri::command]
pub fn store_popup_data(state: tauri::State<Arc<AppState>>, url: String, name: String) {
    *state.popup_data.lock().unwrap() = Some((url, name));
}

#[tauri::command]
pub fn get_popup_data(state: tauri::State<Arc<AppState>>) -> serde_json::Value {
    let data = state.popup_data.lock().unwrap().clone();
    match data {
        Some((url, name)) => serde_json::json!({"url": url, "name": name}),
        None => serde_json::json!({"url": "", "name": ""})
    }
}


#[tauri::command]
pub fn exit_app() { std::process::exit(0); }

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn register_context_menu() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_path = exe.to_string_lossy().to_string();
    let cmd = format!("\"{}\" \"%1\"", exe_path);
    crate::com_shellext::regwrite("Software\\Classes\\*\\shell\\SuShare", "", "\u{901a}\u{8fc7} Su! \u{5206}\u{4eab}");
    crate::com_shellext::regwrite("Software\\Classes\\*\\shell\\SuShare", "MultiSelectModel", "Player");
    crate::com_shellext::regwrite("Software\\Classes\\*\\shell\\SuShare", "Icon", &exe_path);
    crate::com_shellext::regwrite("Software\\Classes\\*\\shell\\SuShare\\command", "", &cmd);
    crate::com_shellext::register_shell_ext(false);
    Ok("ok".into())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn unregister_context_menu() -> Result<String, String> {
    crate::com_shellext::regdelete("Software\\Classes\\*\\shell\\SuShare");
    crate::com_shellext::register_shell_ext(false);
    Ok("ok".into())
}


#[tauri::command]
pub fn close_window(window: tauri::Window, state: tauri::State<Arc<AppState>>) {
    let tray_mode = *state.tray_mode.lock().unwrap();
    if tray_mode {
        if let Err(e) = window.hide() { eprintln!("[Su!] hide failed: {}", e); }
    } else {
        window.close().ok();
    }
}

#[tauri::command]
pub fn get_port(state: tauri::State<Arc<AppState>>) -> u16 {
    state.port
}

#[tauri::command]
pub fn set_sound_settings(state: tauri::State<Arc<AppState>>, enabled: bool, name: String) {
    *state.sound_enabled.lock().unwrap() = enabled;
    *state.sound_name.lock().unwrap() = name;
}


#[tauri::command]
pub fn reset_defaults(state: tauri::State<Arc<AppState>>) -> Result<String, String> {
    if let Ok(mut rx) = state.received.lock() { rx.clear(); }
    if let Ok(mut files) = state.files.lock() { files.clear(); }
    if let Ok(mut bundles) = state.bundles.lock() { bundles.clear(); }
    *state.tray_mode.lock().map_err(|e| e.to_string())? = false;
    *state.clear_on_close.lock().map_err(|e| e.to_string())? = false;
    *state.sound_enabled.lock().map_err(|e| e.to_string())? = true;
    *state.sound_name.lock().map_err(|e| e.to_string())? = "鎶曢€?".into();
    *state.upload_batch.lock().map_err(|e| e.to_string())? = 0;
    *state.batch_expected.lock().map_err(|e| e.to_string())? = 0;
    *state.batch_received_count.lock().map_err(|e| e.to_string())? = 0;
    *state.last_upload.lock().map_err(|e| e.to_string())? = std::time::Instant::now();
    crate::state::save_received(state.inner());
    crate::state::save_shares(state.inner());
    let _ = std::fs::remove_file(state.data_dir.join("received.json"));
    let _ = std::fs::remove_file(state.data_dir.join("shares.json"));
    let _ = std::fs::remove_file(state.data_dir.join("bundles.json"));
    println!("[Su!] defaults reset - all config deleted");
    Ok("ok".into())
}
#[tauri::command]
pub fn set_download_dir(state: tauri::State<Arc<AppState>>, path: String) -> Result<(), String> {
    *state.downloads_dir.lock().map_err(|e| e.to_string())? = std::path::PathBuf::from(&path);
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_download_dir(state: tauri::State<Arc<AppState>>) -> String {
    state.downloads_dir.lock().map(|d| d.to_string_lossy().to_string()).unwrap_or_default()
}

#[tauri::command]
pub fn set_clear_on_close(state: tauri::State<Arc<AppState>>, enabled: bool) {
    *state.clear_on_close.lock().unwrap() = enabled;
}

#[tauri::command]
pub fn get_lang(state: tauri::State<Arc<AppState>>) -> String {
    state.lang.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_lang(state: tauri::State<Arc<AppState>>, lang: String) {
    *state.lang.lock().unwrap() = lang;
}

#[tauri::command]
pub fn get_mobile_lang_mode(state: tauri::State<Arc<AppState>>) -> String {
    state.mobile_lang_mode.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_mobile_lang_mode(state: tauri::State<Arc<AppState>>, mode: String) {
    *state.mobile_lang_mode.lock().unwrap() = mode;
}

#[tauri::command]
pub fn confirm_upload(id: String, accepted: bool, state: tauri::State<Arc<AppState>>) {
    if let Ok(mut map) = state.pending_confirmations.lock() {
        if let Some(tx) = map.remove(&id) {
            let _ = tx.send(accepted);
        }
    }
}

#[tauri::command]
pub fn get_auto_receive(state: tauri::State<Arc<AppState>>) -> bool {
    state.auto_receive.lock().map(|g| *g).unwrap_or(true)
}

#[tauri::command]
pub fn set_auto_receive(enable: bool, state: tauri::State<Arc<AppState>>) {
    *state.auto_receive.lock().unwrap() = enable;
}

#[tauri::command]
pub fn get_auto_destroy(state: tauri::State<Arc<AppState>>) -> bool {
    state.auto_destroy.lock().map(|g| *g).unwrap_or(false)
}

#[tauri::command]
pub fn set_auto_destroy(enable: bool, state: tauri::State<Arc<AppState>>) {
    *state.auto_destroy.lock().unwrap() = enable;
    let path = state.data_dir.join("auto_destroy.cfg");
    let _ = std::fs::write(&path, if enable { "1" } else { "0" });
}

use dirs;

#[tauri::command]
pub fn get_autostart(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let path = state.data_dir.join("autostart.cfg");
    Ok(path.exists() && std::fs::read_to_string(&path).map(|s| s.trim() == "1").unwrap_or(false))
}


#[tauri::command]
pub fn is_context_menu_registered() -> bool {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("Su");
    data_dir.join("ctx_registered.cfg").exists()
}

#[tauri::command]
pub fn set_autostart(enable: bool, state: tauri::State<Arc<AppState>>) -> Result<(), String> {
    let path = state.data_dir.join("autostart.cfg");
    std::fs::write(&path, if enable { "1" } else { "0" }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_page(name: String) -> Result<String, String> {
    match name.as_str() {
        "share" => Ok(include_str!("../../src/pages/share.html").to_string()),
        "received" => Ok(include_str!("../../src/pages/received.html").to_string()),
        "settings" => Ok(include_str!("../../src/pages/settings.html").to_string()),
        "settings-software" => Ok(include_str!("../../src/pages/settings-software.html").to_string()),
        "settings-security" => Ok(include_str!("../../src/pages/settings-security.html").to_string()),
        "settings-receive" => Ok(include_str!("../../src/pages/settings-receive.html").to_string()),
        "settings-notification" => Ok(include_str!("../../src/pages/settings-notification.html").to_string()),
        "settings-appearance" => Ok(include_str!("../../src/pages/settings-appearance.html").to_string()),
        _ => Err(format!("Unknown page: {}", name)),
    }
}

