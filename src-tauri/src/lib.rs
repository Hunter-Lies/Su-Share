mod state;
mod sound;
mod utils;
mod qr;
mod http;
mod commands;
#[cfg(target_os = "windows")]
mod com_shellext;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::Manager;
use tauri::Emitter;
use tauri::Listener;
use rand::Rng;
use state::{AppState, FileEntry};

/// Try to find and connect to an existing instance. Returns the port if one is found.
fn find_existing_instance(data_dir: &PathBuf) -> Option<u16> {
    let lock_path = data_dir.join("instance.port");
    if !lock_path.exists() { return None; }
    let port_str = std::fs::read_to_string(&lock_path).ok()?;
    let port: u16 = port_str.trim().parse().ok()?;
    // Try connecting to verify it's alive
    if std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().ok()?,
        std::time::Duration::from_millis(500),
    ).is_ok() {
        println!("[Su!] found existing instance on port {}", port);
        Some(port)
    } else {
        // Stale lock file
        let _ = std::fs::remove_file(&lock_path);
        None
    }
}

/// Forward CLI paths to an existing instance
fn forward_to_instance(port: u16, paths: &[String]) -> bool {
    let body = paths.join("\n");
    match std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        std::time::Duration::from_millis(1000),
    ) {
        Ok(mut stream) => {
            use std::io::Write;
            let request = format!(
                "POST /cli HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                port, body.len(), body
            );
            stream.write_all(request.as_bytes()).is_ok()
        }
        Err(_) => false,
    }
}

pub fn run() {
    // Determine data dir early for instance detection
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Su");
    std::fs::create_dir_all(&data_dir).ok();

    // CLI args (from context menu or command line)
    let args: Vec<String> = std::env::args().skip(1).filter(|a| !a.starts_with("--")).collect();

    // Check for existing instance
    if let Some(port) = find_existing_instance(&data_dir) {
        if !args.is_empty() {
            println!("[Su!] forwarding {} path(s) to existing instance", args.len());
            forward_to_instance(port, &args);
        }
        // Exit - the existing instance handles everything
        std::process::exit(0);
    }

    let lan_ip = utils::detect_lan_ip();
    let port = {
        let mut p = 52035u16;
        let mut found = false;
        for offset in 0..11 {
            let test_port = 52035 + offset;
            if std::net::TcpListener::bind(format!("0.0.0.0:{}", test_port)).is_ok() {
                p = test_port; found = true; break;
            }
        }
        if !found { p = rand::thread_rng().gen_range(49152..65535); }
        p
    };

    // Write instance lock
    let _ = std::fs::write(data_dir.join("instance.port"), port.to_string());

    let dl = dirs::download_dir()
        .unwrap_or_else(|| dirs::document_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Su");
    std::fs::create_dir_all(&dl).ok();

    let app_state = Arc::new(AppState {
        files: std::sync::Mutex::new(std::collections::HashMap::new()),
        bundles: std::sync::Mutex::new(std::collections::HashMap::new()),
        received: std::sync::Mutex::new(Vec::new()),
        port, lan_ip: lan_ip.clone(),
        downloads_dir: std::sync::Mutex::new(dl), data_dir: data_dir.clone(),
        app_handle: std::sync::Mutex::new(None),
        tray_mode: std::sync::Mutex::new(false),
        pending_cli_paths: std::sync::Mutex::new(Vec::new()),
        popup_data: std::sync::Mutex::new(None),
        upload_batch: std::sync::Mutex::new(0),
        last_upload: std::sync::Mutex::new(std::time::Instant::now()),
        sound_enabled: std::sync::Mutex::new(true),
        sound_name: std::sync::Mutex::new("u6295u9012".into()),
        batch_expected: std::sync::Mutex::new(0),
        batch_received_count: std::sync::Mutex::new(0),
        clear_on_close: std::sync::Mutex::new(false),
        lang: std::sync::Mutex::new("zh-CN".into()),
        mobile_lang_mode: std::sync::Mutex::new("server".into()),
    });
    state::load_received(&app_state);
    state::load_shares(&app_state);

    let ss = Arc::clone(&app_state);
    thread::spawn(move || http::start_http_server(ss));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::share_files, commands::get_server_info, commands::stop_share,
            commands::clear_all_shares, commands::clear_received, commands::generate_qr,
            commands::pick_files, commands::get_send_qr, commands::pick_folder,
            #[cfg(target_os = "windows")] commands::create_shortcut, commands::resize_window, commands::minimize_window,
            commands::toggle_maximize, commands::close_window, commands::set_tray_mode,
            commands::get_cli_paths, commands::store_popup_data,
            commands::get_popup_data,
            commands::exit_app, commands::register_context_menu,
            commands::unregister_context_menu, commands::set_sound_settings, commands::set_download_dir, commands::get_download_dir,
            commands::set_clear_on_close,
            commands::get_lang, commands::set_lang,
            commands::get_mobile_lang_mode, commands::set_mobile_lang_mode, commands::reset_defaults,commands::get_active_shares,
            commands::get_received_files,
            commands::open_path, commands::open_folder, commands::get_port,
        ])
        .setup(move |app| {
            let state = app.state::<Arc<AppState>>();
            *state.app_handle.lock().unwrap() = Some(app.handle().clone());

            // Listen for show-qr-popup events (from forwarded CLI paths)
            let app_handle = app.handle().clone();
            
            app.listen("show-qr-popup", move |_event| {
                // Show main window and let frontend modal handle QR display
                if let Some(window) = app_handle.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            });

            // Build system tray
            let show = MenuItemBuilder::with_id("show", "显示 Su!").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app).item(&show).item(&quit).build()?;
            let state_for_tray = Arc::clone(&state);
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "quit" => { if *state_for_tray.clear_on_close.lock().unwrap() { state_for_tray.received.lock().unwrap().clear(); crate::state::save_received(&state_for_tray); } app.exit(0); }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up, ..
                    } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            
            // Apply acrylic/blur glass effect to window
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::Graphics::Dwm::{
                    DwmSetWindowAttribute, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
                };
                use windows::Win32::Foundation::{HWND, BOOL};
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(wh) = window.window_handle() {
                        if let RawWindowHandle::Win32(h) = wh.as_raw() {
                            let hwnd = HWND(h.hwnd.get() as *mut std::ffi::c_void);
                            let backdrop_type = 4i32; // DWMSBT_TABBEDWINDOW = Acrylic
                            let _ = unsafe {
                                DwmSetWindowAttribute(
                                    hwnd,
                                    DWMWA_SYSTEMBACKDROP_TYPE,
                                    &backdrop_type as *const _ as *const _,
                                    std::mem::size_of::<i32>() as u32,
                                )
                            };
                            let dark: BOOL = BOOL::from(true);
                            let _ = unsafe {
                                DwmSetWindowAttribute(
                                    hwnd,
                                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                                    &dark as *const _ as *const _,
                                    std::mem::size_of::<BOOL>() as u32,
                                )
                            };
                            println!("[Su!] acrylic glass effect applied");
                        }
                    }
                }
            }
// CLI launch from context menu: share directly and show popup
            if !args.is_empty() {
                println!("[Su!] CLI launch with {} path(s)", args.len());
                if let Some(main_win) = app.get_webview_window("main") {
                    main_win.hide().ok();
                }

                let mut map = state.files.lock().unwrap();
                let mut file_ids: Vec<String> = Vec::new();
                let mut shared_names: Vec<String> = Vec::new();
                for p in &args {
                    let pb = PathBuf::from(p);
                    if !pb.exists() { continue; }
                    let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
                    let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                    let fid: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
                    let fid = fid.to_lowercase();
                    map.insert(fid.clone(), FileEntry { name: name.clone(), path: p.clone(), size });
                    file_ids.push(fid);
                    shared_names.push(name.clone());
                }
                drop(map);

                let bundle_id: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
                let bundle_id = bundle_id.to_lowercase();
                let bundle_url = format!("http://{}:{}/s/{}", state.lan_ip, state.port, bundle_id);
                state.bundles.lock().unwrap().insert(bundle_id.clone(), file_ids);
                state::save_shares(&state);
                let popup_name = if shared_names.len() == 1 { shared_names[0].clone() }
                    else { format!("{} files", shared_names.len()) };
                *state.popup_data.lock().unwrap() = Some((bundle_url.clone(), popup_name.clone()));

                // Emit share-added so frontend can render it in the list
                {
                    let share_data: Vec<serde_json::Value> = shared_names.iter().enumerate().map(|(i, n)| {
                        let sz = std::fs::metadata(&args[i]).map(|m| m.len()).unwrap_or(0);
                        serde_json::json!({"name": n, "size": sz})
                    }).collect();
                    let _ = app.emit("share-added", serde_json::json!({
                        "id": bundle_id,
                        "name": popup_name,
                        "url": bundle_url,
                        "files": share_data
                    }));
                }

                // Show main window — frontend will display QR via popup_data
                if let Some(main_win) = app.get_webview_window("main") {
                    main_win.show().ok();
                    main_win.set_focus().ok();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Su! launch failed");
}



