use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize)]
pub struct SharedFile { pub id: String, pub name: String, pub size: u64, pub url: String, pub files: Vec<BundleFileInfo> }

#[derive(Clone, Serialize)]
pub struct BundleFileInfo { pub name: String, pub size: u64 }

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceivedFile { pub name: String, pub size: u64, pub time: String, pub path: String, pub exists: bool, #[serde(default)] pub batch: u64, #[serde(default)] pub device: String }

#[derive(Clone, Serialize, Deserialize)]
pub struct FileEntry { pub name: String, pub path: String, pub size: u64 }

#[derive(Clone, Serialize)]
pub struct ServerInfo { pub url: String, pub lan_ip: String, pub port: u16, pub send_url: String }

pub struct AppState {
    pub files: Mutex<HashMap<String, FileEntry>>,
    pub bundles: Mutex<HashMap<String, Vec<String>>>,
    pub received: Mutex<Vec<ReceivedFile>>,
    pub clear_on_close: Mutex<bool>,
    pub app_handle: Mutex<Option<tauri::AppHandle>>,
    pub port: u16,
    pub lan_ip: String,
    pub downloads_dir: Mutex<PathBuf>,
    pub data_dir: PathBuf,
    pub tray_mode: Mutex<bool>,
    pub pending_cli_paths: Mutex<Vec<String>>,
    pub popup_data: Mutex<Option<(String, String)>>,
    pub lang: Mutex<String>,
    pub mobile_lang_mode: Mutex<String>,
    pub upload_batch: Mutex<u64>,
    pub last_upload: Mutex<Instant>,
    pub sound_enabled: Mutex<bool>,
    pub sound_name: Mutex<String>,
    pub batch_expected: Mutex<u64>,
    pub batch_received_count: Mutex<u64>,
    pub auto_receive: Mutex<bool>,
    pub auto_destroy: Mutex<bool>,
    pub pending_confirmations: Mutex<std::collections::HashMap<String, std::sync::mpsc::Sender<bool>>>,
}

pub fn save_received(state: &Arc<AppState>) {
    if let Ok(rx) = state.received.lock() {
        let path = state.data_dir.join("received.json");
        if let Ok(json) = serde_json::to_string_pretty(&*rx) {
            let _ = std::fs::write(&path, json);
        }
    }
}

pub fn load_received(state: &Arc<AppState>) {
    let path = state.data_dir.join("received.json");
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(items) = serde_json::from_str::<Vec<ReceivedFile>>(&data) {
                if let Ok(mut rx) = state.received.lock() {
                    let items: Vec<ReceivedFile> = items.into_iter().map(|mut item| {
                        item.exists = std::path::Path::new(&item.path).exists();
                        item
                    }).collect();
                    *rx = items;
                    println!("[Su!] loaded {} received records", rx.len());
                }
            }
        }
    }
}


pub fn save_shares(state: &Arc<AppState>) {
    let data_dir = &state.data_dir;
    if let Ok(files) = state.files.lock() {
        let path = data_dir.join("shares.json");
        if let Ok(json) = serde_json::to_string_pretty(&*files) {
            let _ = std::fs::write(&path, json);
        }
    }
    if let Ok(bundles) = state.bundles.lock() {
        let path = data_dir.join("bundles.json");
        if let Ok(json) = serde_json::to_string_pretty(&*bundles) {
            let _ = std::fs::write(&path, json);
        }
    }
}

pub fn load_shares(state: &Arc<AppState>) {
    let data_dir = &state.data_dir;
    let files_path = data_dir.join("shares.json");
    if files_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&files_path) {
            if let Ok(items) = serde_json::from_str::<HashMap<String, FileEntry>>(&data) {
                if let Ok(mut files) = state.files.lock() {
                    *files = items;
                }
            }
        }
    }
    let bundles_path = data_dir.join("bundles.json");
    if bundles_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&bundles_path) {
            if let Ok(items) = serde_json::from_str::<HashMap<String, Vec<String>>>(&data) {
                if let Ok(mut bundles) = state.bundles.lock() {
                    *bundles = items;
                }
            }
        }
    }
}




