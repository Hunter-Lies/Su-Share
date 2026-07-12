    use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tauri::Emitter;
use tauri::Manager;
use crate::state::{AppState, FileEntry, ReceivedFile, save_shares};
use crate::sound;

use crate::utils::{mime_for, make_header, fmt_size, fmt_speed, respond, not_found};
fn inject_lang(html: &str, lang: &str, mode: &str) -> String { html.replacen("</head>", &format!(r###"<script>window.__SU_LANG="{0}";window.__SU_LANG_MODE="{1}"</script></head>"###, lang, mode), 1) }
fn serve_html(req: tiny_http::Request, html: &str) {
    respond(req, 200, "text/html; charset=utf-8", html.as_bytes());
}

pub fn start_http_server(state: Arc<AppState>) {
    let port = state.port;
    println!("[Su!] HTTP on http://{}:{}", state.lan_ip, port);
    let server = match tiny_http::Server::http(format!("0.0.0.0:{}", port)) {
        Ok(s) => s,
        Err(e) => { eprintln!("[Su!] bind failed: {}", e); return; }
    };
    println!("[Su!] ready");
    loop {
        if let Ok(req) = server.recv() {
            let url = req.url().to_string();
            let method = req.method().to_string();
            let peer = req.remote_addr().map(|a| a.to_string()).unwrap_or_else(|| "?".into());
            let state = Arc::clone(&state);
            let t0 = Instant::now();
            println!("[Su!] <- {} {} from {}", method, url, peer);
            thread::spawn(move || {
                let r = dispatch(req, &url, &method, &state);
                let el = t0.elapsed();
                if let Some((bytes, st, fname)) = r {
                    println!("[Su!] -> {} {} | {} | {} | {}ms", st, fname, fmt_size(bytes), fmt_speed(bytes, el), el.as_millis());
                } else { println!("[Su!] -> ({}ms)", el.as_millis()); }
            });
        }
    }
}

fn dispatch(req: tiny_http::Request, url: &str, method: &str, state: &Arc<AppState>) -> Option<(u64, u16, String)> {
    if method == "OPTIONS" {
        println!("[Su!] dispatch: OPTIONS {}", url);
        respond(req, 204, "text/plain", b"");
        None
    } else if url == "/" && method == "GET" {
        println!("[Su!] dispatch: / -> send page");
        let lang = state.lang.lock().unwrap().clone(); let mode = state.mobile_lang_mode.lock().unwrap().clone(); let html = inject_lang(include_str!("../web/send.html"), &lang, &mode); serve_html(req, &html);
        None
    } else if url == "/upload-start" && method == "POST" {
        println!("[Su!] dispatch: /upload-start");
        Some(handle_upload_start(req, state))
    } else if url.starts_with("/upload") && method == "POST" {
        println!("[Su!] dispatch: /upload");
        Some(handle_upload(req, url, state))
    } else if url.starts_with("/cli") && method == "POST" {
        println!("[Su!] dispatch: /cli forward");
        Some(handle_cli_forward(req, state))
    } else if let Some(rest) = url.strip_prefix("/s/") {
        let rest = rest.trim_end_matches('/');
        println!("[Su!] dispatch: /s/{} bundle page", rest);
        if method == "GET" { Some(serve_bundle_page(req, rest, state)) }
        else { not_found(req); None }
    } else if let Some(rest) = url.strip_prefix("/d/") {
        let is_dl = rest.contains("dl=1");
        let rest = rest.split('?').next().unwrap_or(rest);
        println!("[Su!] dispatch: /d/{} is_dl={}", rest, is_dl);
        if method == "GET" && is_dl {
            Some(serve_raw_file(req, rest, state))
        } else { Some(serve_download_page(req, rest, state)) }
    } else if url == "/i18n.js" && method == "GET" {
        println!("[Su!] dispatch: /i18n.js");
        let len = include_str!("../web/i18n.js").len() as u64;
        respond(req, 200, "application/javascript; charset=utf-8", include_str!("../web/i18n.js").as_bytes());
        Some((len, 200, "i18n.js".into()))
    } else if let Some(rest) = url.strip_prefix("/fonts/") {
        println!("[Su!] dispatch: /fonts/{}", rest);
        return Some(serve_font(req, rest))
    } else if let Some(rest) = url.strip_prefix("/v/") {
        println!("[Su!] dispatch: /v/{}", rest);
        Some(serve_raw_file(req, rest, state))
    } else {
        println!("[Su!] dispatch: 404 for {}", url);
        not_found(req);
        None
    }
}


fn guess_device(req: &tiny_http::Request) -> String {
    let ua = req.headers().iter()
        .find(|h| h.field.equiv("User-Agent"))
        .map(|h| h.value.to_string())
        .unwrap_or_default();
    let ua_lower = ua.to_lowercase();
    if ua_lower.contains("iphone") || ua_lower.contains("ipad") { "iPhone".into() }
    else if ua_lower.contains("android") { "Android".into() }
    else if ua_lower.contains("windows") { "Windows".into() }
    else if ua_lower.contains("macintosh") || ua_lower.contains("mac os") { "Mac".into() }
    else if ua_lower.contains("linux") { "Linux".into() }
    else { "Unknown".into() }
}

fn handle_upload_start(mut req: tiny_http::Request, state: &Arc<AppState>) -> (u64, u16, String) {
    let mut body = String::new();
    req.as_reader().read_to_string(&mut body).ok();
    let count: u64 = body.trim().parse().unwrap_or(0);
    println!("[Su!] upload-start: expecting {} files", count);
    *state.batch_expected.lock().unwrap() = count;
    *state.batch_received_count.lock().unwrap() = 0;

    let auto = *state.auto_receive.lock().unwrap();
    if !auto {
        // Generate pending ID and emit event to frontend
        let pending_id: String = std::iter::repeat_with(|| fastrand::alphanumeric()).take(10).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        state.pending_confirmations.lock().unwrap().insert(pending_id.clone(), tx);

        let device = guess_device(&req);
        // Focus window and emit event
        if let Some(app) = state.app_handle.lock().ok().and_then(|h| h.clone()) {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = app.emit("upload-requested", serde_json::json!({
                "id": pending_id,
                "device": device,
                "count": count,
            }));
        }
        println!("[Su!] upload-start: waiting for user confirmation...");
        let accepted = rx.recv().unwrap_or(false);
        state.pending_confirmations.lock().unwrap().remove(&pending_id);
        if !accepted {
            println!("[Su!] upload-start: rejected by user");
            respond(req, 403, "text/plain", b"rejected");
            return (0, 403, String::new());
        }
        println!("[Su!] upload-start: accepted by user");
    }

    respond(req, 200, "text/plain", b"ok");
    (0, 200, String::new())
}



fn handle_upload(mut req: tiny_http::Request, url: &str, state: &Arc<AppState>) -> (u64, u16, String) {

    let filename = url.split('?').nth(1)
        .and_then(|qs| qs.split('&').find(|p| p.starts_with("name=")))
        .map(|p| p[5..].to_string())
        .unwrap_or_else(|| "received_file".into());
    let filename = urlencoding::decode(&filename).unwrap_or_else(|_| "received_file".into()).into_owned();
    println!("[Su!] upload start: {}", filename);
    // Detect device from User-Agent
    let ua = req.headers().iter()
        .find(|h| h.field.equiv("User-Agent"))
        .map(|h| h.value.as_str().to_string())
        .unwrap_or_default();
    let device = if ua.contains("iPhone") || ua.contains("iPad") { "iPhone" }
        else if ua.contains("Android") { "Android" }
        else if ua.contains("Windows") { "Windows" }
        else if ua.contains("Macintosh") || ua.contains("Mac OS") { "Mac" }
        else if ua.contains("Linux") { "Linux" }
        else { "Unknown" };
    println!("[Su!] upload from {} (UA: {})", device, &ua[..ua.len().min(60)]);


    let mut dest = state.downloads_dir.lock().unwrap().join(&filename);
    if dest.exists() {
        let stem = dest.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
        let ext = dest.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let dl = state.downloads_dir.lock().unwrap();
        let parent = dest.parent().unwrap_or(&dl);
        let mut counter = 2u32;
        loop {
            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            };
            let candidate = parent.join(&new_name);
            if !candidate.exists() { dest = candidate; break; }
            counter += 1;
        }
        println!("[Su!] upload rename: {} -> {}", filename,
            dest.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());
    }

    let mut file = match std::fs::File::create(&dest) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[Su!] upload create file failed: {} ({})", filename, e);
            respond(req, 500, "text/plain", b"create failed");
            return (0, 500, filename);
        }
    };

    let len = match std::io::copy(&mut req.as_reader(), &mut file) {
        Ok(n) => {
            println!("[Su!] received: {} ({}) -> {}", filename, fmt_size(n), dest.display());
            n
        }
        Err(e) => {
            eprintln!("[Su!] upload read failed: {} ({})", filename, e);
            let _ = std::fs::remove_file(&dest);
            respond(req, 500, "text/plain", b"read failed");
            return (0, 500, filename);
        }
    };

        if let Ok(mut rx) = state.received.lock() {
        // Batch + sound: lock both atomically to prevent race
        let mut upload_batch = state.upload_batch.lock().unwrap();
        let mut last = state.last_upload.lock().unwrap();
        let is_new_batch = last.elapsed().as_secs() >= 2;
        if is_new_batch { *upload_batch += 1; }
        let current_batch = *upload_batch;
        *last = Instant::now();
        drop(last);
        drop(upload_batch);

        use std::time::SystemTime;
        let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let secs = t.as_secs() + 8 * 3600;
        let days = secs / 86400;
        let (mut y, mut rem) = (1970u64, days);
        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let days_in_year: u64 = if leap { 366 } else { 365 };
            if rem < days_in_year { break; }
            rem -= days_in_year;
            y += 1;
        }
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let md: [u64; 12] = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut mo = 1u64;
        for i in 0..12 {
            if rem < md[i] { mo = (i + 1) as u64; break; }
            rem -= md[i];
        }
        let d_num = rem + 1;
        let h = (secs / 3600) % 24;
        let m = (secs / 60) % 60;
        rx.push(ReceivedFile {
            name: filename.clone(), size: len,
            time: format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d_num, h, m),
            path: dest.to_string_lossy().to_string(), exists: true, batch: current_batch, device: device.to_string(),
        });
        drop(rx);
        crate::state::save_received(state);
        // Batch-complete sound: play when all expected files arrived
        let expected = *state.batch_expected.lock().unwrap();
        if expected > 0 {
            let mut cnt = state.batch_received_count.lock().unwrap();
            *cnt += 1;
            println!("[Su!] batch progress: {}/{}", *cnt, expected);
            if *cnt >= expected {
                drop(cnt);
                sound::play_received_sound(state);
                // Emit batch-complete for popup
                if let Some(app) = state.app_handle.lock().ok().and_then(|h| h.clone()) {
                    let _ = app.emit("batch-complete", serde_json::json!({
                        "device": device,
                        "count": expected,
                    }));
                    // Bring window to front
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                *state.batch_expected.lock().unwrap() = 0;
            }
        } else {
            sound::play_received_sound(state);
        }
    }
    println!("[Su!] upload done: {} ({} bytes)", filename, len);
    if let Err(e) = req.respond(tiny_http::Response::from_data(b"OK".to_vec())
        .with_status_code(200)
        .with_header(make_header("Content-Type", "text/plain; charset=utf-8"))
        .with_header(make_header("Content-Length", "2"))
        .with_header(make_header("Connection", "close"))
        .with_header(make_header("Access-Control-Allow-Origin", "*"))) {
        eprintln!("[Su!] respond error: {}", e);
    }

    if let Some(app) = state.app_handle.lock().ok().and_then(|h| h.clone()) {
        let _ = app.emit("file-received", serde_json::json!({
            "name": filename,
            "size": len,
            "path": dest.to_string_lossy(),
            "device": device
        }));
    }

    (len, 200, filename)
}

fn serve_download_page(req: tiny_http::Request, raw_id: &str, state: &Arc<AppState>) -> (u64, u16, String) {
    println!("[Su!] download_page: id={}", raw_id);
    let sid = raw_id.trim_end_matches('/');
    let guard = match state.files.lock() {
        Ok(g) => g,
        Err(_) => { respond(req, 500, "text/plain", b"err"); return (0, 500, "".into()); }
    };
    let entry = match guard.get(sid) {
        Some(e) => e.clone(),
        None => {
            let body = "<!DOCTYPE html><html lang=\"zh-CN\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Expired</title><style>body{font-family:-apple-system,sans-serif;background:#f5f5f5;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0}.card{background:#fff;border-radius:16px;padding:48px 32px;text-align:center;box-shadow:0 4px 24px rgba(0,0,0,.08);max-width:360px}h2{color:#e74c3c;margin-bottom:8px}p{color:#666}</style></head><body><div class=\"card\"><h2>Share Expired</h2><p>Ask sender to reshare</p></div></body></html>";
            respond(req, 404, "text/html; charset=utf-8", body.as_bytes());
            return (0, 404, "expired".into());
        }
    };
    let dl = format!("/d/{}?dl=1", sid);
    let lang = state.lang.lock().unwrap().clone(); let mode = state.mobile_lang_mode.lock().unwrap().clone();
    let page = inject_lang(include_str!("../web/download.html"), &lang, &mode)
        .replace("__NAME__", &entry.name)
        .replace("__SIZE__", &fmt_size(entry.size))
        .replace("__URL__", &dl);
    respond(req, 200, "text/html; charset=utf-8", page.as_bytes());
    (0, 200, entry.name)
}

fn serve_raw_file(req: tiny_http::Request, raw_id: &str, state: &Arc<AppState>) -> (u64, u16, String) {
    println!("[Su!] raw_file: id={}", raw_id);
    let sid = raw_id.trim_end_matches('/');
    let guard = match state.files.lock() {
        Ok(g) => g,
        Err(_) => { respond(req, 500, "text/plain", b"err"); return (0, 500, "".into()); }
    };
    let entry = match guard.get(sid) {
        Some(e) => e.clone(),
        None => { respond(req, 404, "text/plain", b"not found"); return (0, 404, "".into()); }
    };
    let mut file = match std::fs::File::open(&entry.path) {
        Ok(f) => f,
        Err(_) => { respond(req, 404, "text/plain", b"not found"); return (0, 404, entry.name); }
    };
    let total_len = match file.metadata() { Ok(m) => m.len(), Err(_) => 0 };
    let safe_name = entry.name.replace('"', "").replace('\\', "");
    let is_ascii = safe_name.chars().all(|c| c.is_ascii() && !c.is_control());
    let cd = if is_ascii {
        format!("attachment; filename=\"{}\"", safe_name)
    } else {
        format!("attachment; filename*=UTF-8''{}", urlencoding::encode(&safe_name))
    };

    let range_header = req.headers().iter().find(|h| h.field.to_string().to_lowercase() == "range");
    let (start, end, status, content_len) = if let Some(rh) = range_header {
        let val = rh.value.to_string();
        if let Some(range_val) = val.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_val.split('-').collect();
            if parts.len() == 2 {
                let range_start: u64 = parts[0].parse().unwrap_or(0);
                let range_end: u64 = if parts[1].is_empty() {
                    total_len.saturating_sub(1)
                } else {
                    parts[1].parse().unwrap_or(total_len.saturating_sub(1))
                };
                let range_end = range_end.min(total_len.saturating_sub(1));
                if range_start <= range_end && range_end < total_len {
                    let cl = range_end - range_start + 1;
                    (range_start, range_end, 206u16, cl)
                } else { (0u64, total_len.saturating_sub(1), 200u16, total_len) }
            } else { (0u64, total_len.saturating_sub(1), 200u16, total_len) }
        } else { (0u64, total_len.saturating_sub(1), 200u16, total_len) }
    } else { (0u64, total_len.saturating_sub(1), 200u16, total_len) };

    if status == 206 {
        use std::io::Seek;
        if file.seek(std::io::SeekFrom::Start(start)).is_err() {
            respond(req, 500, "text/plain", b"seek failed");
            return (0, 500, entry.name);
        }
        let content_range = format!("bytes {}-{}/{}", start, end, total_len);
        req.respond(tiny_http::Response::from_file(file)
            .with_status_code(206)
            .with_header(make_header("Content-Type", &mime_for(&entry.name)))
            .with_header(make_header("Content-Disposition", &cd))
            .with_header(make_header("Content-Length", &content_len.to_string()))
            .with_header(make_header("Content-Range", &content_range))
            .with_header(make_header("Accept-Ranges", "bytes"))
            .with_header(make_header("X-Content-Type-Options", "nosniff"))
            .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
        println!("[Su!] sent range: {} ({}-{})", entry.name, start, end);
        (content_len, 206, entry.name)
    } else {
        req.respond(tiny_http::Response::from_file(file)
            .with_header(make_header("Content-Type", &mime_for(&entry.name)))
            .with_header(make_header("Content-Disposition", &cd))
            .with_header(make_header("Content-Length", &total_len.to_string()))
            .with_header(make_header("Accept-Ranges", "bytes"))
            .with_header(make_header("X-Content-Type-Options", "nosniff"))
            .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
        println!("[Su!] sent: {} ({})", entry.name, fmt_size(total_len));
        (total_len, 200, entry.name)
    }
}

fn serve_bundle_page(req: tiny_http::Request, bundle_id: &str, state: &Arc<AppState>) -> (u64, u16, String) {
    let bundles = state.bundles.lock().unwrap();
    let files = state.files.lock().unwrap();
    if let Some(file_ids) = bundles.get(bundle_id) {
        let count = file_ids.len();
        if count == 1 {
            if let Some(entry) = files.get(&file_ids[0]) {
                let url = format!("/d/{}?dl=1", file_ids[0]);
                let lang = state.lang.lock().unwrap().clone(); let mode = state.mobile_lang_mode.lock().unwrap().clone();
                let html = inject_lang(include_str!("../web/download.html"), &lang, &mode)
                    .replace("__NAME__", &entry.name)
                    .replace("__SIZE__", &fmt_size(entry.size))
                    .replace("__URL__", &url);
                let body = html.into_bytes();
                let len = body.len() as u64;
                req.respond(tiny_http::Response::from_data(body).with_status_code(200)
                    .with_header(make_header("Content-Type", "text/html; charset=utf-8"))
                    .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
                return (len, 200, entry.name.clone());
            }
        }
        // Multiple files
        let mut items = String::new();
        for fid in file_ids.iter() {
            if let Some(entry) = files.get(fid) {
                items.push_str(&format!(
                    r###"<label class="fcard"><input type="checkbox" class="fcb" data-url="/d/{}?dl=1" data-name="{}"><span class="fcard-ck"><i class="fa-regular fa-circle"></i><i class="fa-solid fa-circle-check"></i></span><div class="fcard-icon"><i class="fa-solid fa-file"></i></div><div class="fcard-name">{}</div><div class="fcard-size">{}</div></label>"###,
                    fid, entry.name, entry.name, fmt_size(entry.size)
                ));
            }
        }
        let lang = state.lang.lock().unwrap().clone(); let mode = state.mobile_lang_mode.lock().unwrap().clone(); let html = inject_lang(include_str!("../web/bundle_multi.html"), &lang, &mode).replace("{items}", &items).replace("__CNT__", &count.to_string());
        let body = html.into_bytes();
        let len = body.len() as u64;
        req.respond(tiny_http::Response::from_data(body).with_status_code(200)
            .with_header(make_header("Content-Type", "text/html; charset=utf-8"))
            .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
        (len, 200, format!("bundle {}", bundle_id))
    } else {
        not_found(req);
        (0, 404, String::new())
    }
}

fn serve_font(req: tiny_http::Request, path: &str) -> (u64, u16, String) {
    let mime = if path.ends_with(".css") { "text/css" } else if path.ends_with(".woff2") { "font/woff2" } else { "application/octet-stream" };
    let content = match path {
        "all.min.css" => include_bytes!("../web/fonts/all.min.css").to_vec(),
        "fa-solid-900.woff2" => include_bytes!("../web/fonts/fa-solid-900.woff2").to_vec(),
        "fa-regular-400.woff2" => include_bytes!("../web/fonts/fa-regular-400.woff2").to_vec(),
        "fa-brands-400.woff2" => include_bytes!("../web/fonts/fa-brands-400.woff2").to_vec(),
        _ => { not_found(req); return (0, 404, String::new()); }
    };
    let len = content.len() as u64;
    req.respond(tiny_http::Response::from_data(content).with_status_code(200)
        .with_header(make_header("Content-Type", mime))
        .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
    (len, 200, path.to_string())
}

/// Handle forwarded CLI paths from another instance
fn handle_cli_forward(mut req: tiny_http::Request, state: &Arc<AppState>) -> (u64, u16, String) {
    let mut body = String::new();
    if req.as_reader().read_to_string(&mut body).is_err() {
        respond(req, 400, "text/plain", b"bad request");
        return (0, 400, "".into());
    }
    let paths: Vec<String> = body.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
    // Check if this is a focus request
    if paths.len() == 1 && paths[0] == "__focus__" {
        println!("[Su!] cli focus: bringing window to front");
        if let Some(app) = state.app_handle.lock().ok().and_then(|h| h.clone()) {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        respond(req, 200, "text/plain", b"ok");
        return (0, 200, "".into());
    }
    println!("[Su!] cli forward: {} paths", paths.len());
    if paths.is_empty() {
        respond(req, 200, "text/plain", b"ok");
        return (0, 200, "".into());
    }
    process_cli_paths(state, &paths);
    respond(req, 200, "text/plain", b"ok");
    (0, 200, "".into())
}

pub fn process_cli_paths(state: &Arc<AppState>, paths: &[String]) {
    use rand::Rng;
    let mut map = state.files.lock().unwrap();
    let mut file_ids: Vec<String> = Vec::new();
    let mut shared_names: Vec<String> = Vec::new();
    for p in paths {
        let pb = std::path::PathBuf::from(p);
        if !pb.exists() { continue; }
        let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
        let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let fid: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
        let fid = fid.to_lowercase();
        map.insert(fid.clone(), FileEntry { name: name.clone(), path: p.clone(), size });
        file_ids.push(fid);
        shared_names.push(name);
    }
    drop(map);

    let bundle_id: String = (0..8).map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char).collect();
    let bundle_id = bundle_id.to_lowercase();
    let bundle_url = format!("http://{}:{}/s/{}", state.lan_ip, state.port, bundle_id);
    state.bundles.lock().unwrap().insert(bundle_id.clone(), file_ids);
    let popup_name = if shared_names.len() == 1 {
        shared_names[0].clone()
    } else {
        format!("{} files", shared_names.len())
    };
    *state.popup_data.lock().unwrap() = Some((bundle_url.clone(), popup_name.clone()));
    save_shares(state);

    // Show QR popup + share-added events via Tauri event
    if let Some(app) = state.app_handle.lock().ok().and_then(|h| h.clone()) {
        let _ = app.emit("show-qr-popup", serde_json::json!({
            "url": bundle_url,
            "name": popup_name
        }));
        // Also notify frontend to add this share to the list
        let share_data: Vec<serde_json::Value> = shared_names.iter().enumerate().map(|(i, n)| {
            serde_json::json!({"name": n, "size": std::fs::metadata(&paths[i]).map(|m| m.len()).unwrap_or(0)})
        }).collect();
        let _ = app.emit("share-added", serde_json::json!({
            "id": bundle_id,
            "name": popup_name,
            "url": bundle_url,
            "files": share_data
        }));
    }
}




