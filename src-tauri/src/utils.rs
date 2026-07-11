use std::time::Duration;
use tiny_http;

pub fn detect_lan_ip() -> String {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
        for iface in interfaces {
            let ip = iface.ip().to_string();
            if ip.starts_with("127.") || ip.starts_with("198.18.") || ip.starts_with("198.19.") {
                continue;
            }
            if iface.is_loopback() { continue; }
            if ip.starts_with("192.168.") || ip.starts_with("10.") || ip.starts_with("172.16.")
               || ip.starts_with("172.17.") || ip.starts_with("172.18.") || ip.starts_with("172.19.")
               || ip.starts_with("172.20.") || ip.starts_with("172.21.") || ip.starts_with("172.22.")
               || ip.starts_with("172.23.") || ip.starts_with("172.24.") || ip.starts_with("172.25.")
               || ip.starts_with("172.26.") || ip.starts_with("172.27.") || ip.starts_with("172.28.")
               || ip.starts_with("172.29.") || ip.starts_with("172.30.") || ip.starts_with("172.31.") {
                candidates.push(ip);
            }
        }
    }
    candidates.sort_by(|a, b| {
        let score = |s: &str| {
            if s.starts_with("192.168.") { 0 }
            else if s.starts_with("10.") { 1 }
            else { 2 }
        };
        score(a).cmp(&score(b))
    });
    if let Some(ip) = candidates.first() {
        println!("[Su!] LAN IP: {}", ip);
        return ip.clone();
    }
    match local_ip_address::local_ip() {
        Ok(ip) => {
            let s = ip.to_string();
            if s.starts_with("127.") || s.starts_with("198.18.") || s.starts_with("198.19.") {
                eprintln!("[Su!] all IPs virtual, using 127.0.0.1");
                "127.0.0.1".into()
            } else {
                println!("[Su!] LAN IP (fallback): {}", s);
                s
            }
        }
        Err(e) => { eprintln!("[Su!] no LAN: {}", e); "127.0.0.1".into() }
    }
}

pub fn mime_for(name: &str) -> String {
    mime_guess::from_path(name).first_or_octet_stream().to_string()
}

pub fn make_header(k: &str, v: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(k.as_bytes(), v.as_bytes())
        .unwrap_or_else(|_| tiny_http::Header::from_bytes("X-Fallback".as_bytes(), "1".as_bytes()).unwrap())
}

pub fn fmt_speed(bytes: u64, dur: Duration) -> String {
    let s = dur.as_secs_f64();
    if s < 0.001 { return "N/A".into(); }
    let bps = bytes as f64 / s;
    if bps > 1_000_000.0 { format!("{:.1} MB/s", bps/1_000_000.0) }
    else if bps > 1_000.0 { format!("{:.1} KB/s", bps/1_000.0) }
    else { format!("{:.0} B/s", bps) }
}

pub fn fmt_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1_048_576 { format!("{:.1} KB", bytes as f64/1024.0) }
    else if bytes < 1_073_741_824 { format!("{:.1} MB", bytes as f64/1_048_576.0) }
    else { format!("{:.2} GB", bytes as f64/1_073_741_824.0) }
}

pub fn respond(req: tiny_http::Request, status: u16, ct: &str, body: &[u8]) {
    req.respond(tiny_http::Response::from_data(body.to_vec())
        .with_status_code(status)
        .with_header(make_header("Content-Type", ct))
        .with_header(make_header("Access-Control-Allow-Origin", "*"))).ok();
}

pub fn not_found(req: tiny_http::Request) {
    respond(req, 404, "text/plain", b"not found");
}
