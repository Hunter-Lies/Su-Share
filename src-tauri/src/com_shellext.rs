#![cfg(target_os = "windows")]
// COM Shell Extension - Win11 primary context menu support
// Implements IExplorerCommand, registers via HKCU (no admin needed)

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const S_OK: i32 = 0;
const E_FAIL: i32 = -2147467259i32;
const E_NOINTERFACE: i32 = -2147467262i32;

const CLSID_DATA1: u32 = 0xD4E8F1A2;
const CLSID_DATA2: u16 = 0x3B5C;
const CLSID_DATA3: u16 = 0x4D6E;
const CLSID_DATA4: [u8; 8] = [0x9F, 0x0A, 0x1B, 0x2C, 0x3D, 0x4E, 0x5F, 0x6A];

static mut DLL_HINSTANCE: *mut std::ffi::c_void = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinst: *mut std::ffi::c_void, reason: u32, _reserved: *mut std::ffi::c_void) -> i32 {
    if reason == 1 { DLL_HINSTANCE = hinst; }
    1
}

fn get_dll_dir() -> PathBuf {
    // When running inside su.exe (Tauri), get DLL path from EXE path
    // When loaded by Explorer as COM server, use DLL_HINSTANCE
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    // Check if we're in the EXE (su.exe) or DLL (su_lib.dll)
    let stem = exe.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    if stem == "su_lib" {
        // Running as DLL, use DLL_HINSTANCE
        unsafe {
            let mut buf = vec![0u16; 520];
            let len = GetModuleFileNameW(DLL_HINSTANCE, buf.as_mut_ptr(), buf.len() as u32);
            if len > 0 {
                buf.truncate(len as usize);
                let p = PathBuf::from(OsString::from_wide(&buf));
                return p.parent().map(|x| x.to_path_buf()).unwrap_or(dir);
            }
        }
    }
    dir
}

fn get_dll_path() -> PathBuf {
    get_dll_dir().join("su_lib.dll")
}

fn find_su_exe() -> Option<PathBuf> {
    let dir = get_dll_dir();
    let exe = dir.join("su.exe");
    if exe.exists() { return Some(exe); }
    None
}

#[repr(C)]
struct GuidRaw { data1: u32, data2: u16, data3: u16, data4: [u8; 8] }

#[repr(C)]
struct IUnknownVtbl {
    query_interface: unsafe extern "system" fn(*mut std::ffi::c_void, *const GuidRaw, *mut *mut std::ffi::c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
}

#[repr(C)]
struct IClassFactoryVtbl {
    iunknown: IUnknownVtbl,
    create_instance: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const GuidRaw, *mut *mut std::ffi::c_void) -> i32,
    lock_server: unsafe extern "system" fn(*mut std::ffi::c_void, i32) -> i32,
}

#[repr(C)]
struct IExplorerCommandVtbl {
    iunknown: IUnknownVtbl,
    get_title: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut *mut u16) -> i32,
    get_icon: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut *mut u16) -> i32,
    get_tooltip: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut *mut u16) -> i32,
    get_canonical_name: unsafe extern "system" fn(*mut std::ffi::c_void, *mut GuidRaw) -> i32,
    get_state: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, i32, *mut u32) -> i32,
    invoke: unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> i32,
    get_flags: unsafe extern "system" fn(*mut std::ffi::c_void, *mut u32) -> i32,
    enum_sub_commands: unsafe extern "system" fn(*mut std::ffi::c_void, *mut *mut std::ffi::c_void) -> i32,
}

#[repr(C)]
struct IShellItemArrayVtbl {
    iunknown: IUnknownVtbl,
    bind_to_handler: *const std::ffi::c_void,
    get_property_store: *const std::ffi::c_void,
    get_property_description_list: *const std::ffi::c_void,
    get_attributes: *const std::ffi::c_void,
    get_count: unsafe extern "system" fn(*mut std::ffi::c_void, *mut u32) -> i32,
    get_item_at: unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut *mut std::ffi::c_void) -> i32,
    enum_items: *const std::ffi::c_void,
}

#[repr(C)]
struct IShellItemVtbl {
    iunknown: IUnknownVtbl,
    bind_to_handler: *const std::ffi::c_void,
    get_parent: *const std::ffi::c_void,
    get_display_name: unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut *mut u16) -> i32,
    get_attributes: *const std::ffi::c_void,
    compare: *const std::ffi::c_void,
}

const SIGDN_FILESYSPATH: u32 = 0x80058000;

struct ClassFactory { vtbl: &'static IClassFactoryVtbl, refcount: AtomicU32 }
struct ExplorerCommand { vtbl: &'static IExplorerCommandVtbl, refcount: AtomicU32 }

extern "system" {
    fn GetModuleFileNameW(hModule: *mut std::ffi::c_void, lpFilename: *mut u16, nSize: u32) -> u32;
    fn CoTaskMemAlloc(size: usize) -> *mut std::ffi::c_void;
    fn CoTaskMemFree(ptr: *mut std::ffi::c_void);
}

fn alloc_wide(s: &str) -> *mut u16 {
    let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let ptr = CoTaskMemAlloc(wide.len() * 2) as *mut u16;
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        }
        ptr
    }
}

// ---- IUnknown ----
unsafe extern "system" fn qi(_this: *mut std::ffi::c_void, riid: *const GuidRaw, ppv: *mut *mut std::ffi::c_void) -> i32 {
    if ppv.is_null() { return -2147024809i32; }
    *ppv = std::ptr::null_mut();
    if riid.is_null() { return E_NOINTERFACE; }
    let riid = &*riid;
    // IUnknown: {00000000-0000-0000-C000-000000000046}
    let is_iunknown = riid.data1 == 0 && riid.data2 == 0 && riid.data3 == 0
        && riid.data4[0] == 0xC0 && riid.data4[1] == 0x00;
    // IExplorerCommand: {a08ce4d0-fa25-44ab-b57c-c7b1c323e0b9}
    let is_iec = riid.data1 == 0xa08ce4d0 && riid.data2 == 0xfa25 && riid.data3 == 0x44ab;
    if is_iunknown || is_iec {
        *ppv = _this;
        add_ref(_this);
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn add_ref(this: *mut std::ffi::c_void) -> u32 {
    if this.is_null() { return 0; }
    let rc = (this as *mut u8).add(std::mem::size_of::<*const std::ffi::c_void>()) as *const AtomicU32;
    (*rc).fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn release(this: *mut std::ffi::c_void) -> u32 {
    if this.is_null() { return 0; }
    let rc = (this as *mut u8).add(std::mem::size_of::<*const std::ffi::c_void>()) as *const AtomicU32;
    (*rc).fetch_sub(1, Ordering::SeqCst) - 1
}

// ---- ClassFactory ----
static CF_VTBL: IClassFactoryVtbl = IClassFactoryVtbl {
    iunknown: IUnknownVtbl { query_interface: qi, add_ref, release },
    create_instance: cf_create,
    lock_server: cf_lock,
};

static mut CLASS_FACTORY: ClassFactory = ClassFactory { vtbl: &CF_VTBL, refcount: AtomicU32::new(1) };

unsafe extern "system" fn cf_create(_this: *mut std::ffi::c_void, _outer: *mut std::ffi::c_void, riid: *const GuidRaw, ppv: *mut *mut std::ffi::c_void) -> i32 {
    if ppv.is_null() { return -2147024809i32; }
    *ppv = std::ptr::null_mut();
    let cmd = Box::new(ExplorerCommand { vtbl: &EC_VTBL, refcount: AtomicU32::new(1) });
    let ptr = Box::into_raw(cmd) as *mut std::ffi::c_void;
    let hr = qi(ptr, riid, ppv);
    if hr < 0 { drop(Box::from_raw(ptr as *mut ExplorerCommand)); }
    hr
}

unsafe extern "system" fn cf_lock(_this: *mut std::ffi::c_void, _lock: i32) -> i32 { S_OK }

// ---- ExplorerCommand ----
static EC_VTBL: IExplorerCommandVtbl = IExplorerCommandVtbl {
    iunknown: IUnknownVtbl { query_interface: qi, add_ref, release },
    get_title: ec_get_title,
    get_icon: ec_get_icon,
    get_tooltip: ec_get_tooltip,
    get_canonical_name: ec_get_canonical_name,
    get_state: ec_get_state,
    invoke: ec_invoke,
    get_flags: ec_get_flags,
    enum_sub_commands: ec_enum_sub,
};

unsafe extern "system" fn ec_get_title(_this: *mut std::ffi::c_void, _psiia: *mut std::ffi::c_void, ppsz: *mut *mut u16) -> i32 {
    if ppsz.is_null() { return -2147024809i32; }
    *ppsz = alloc_wide("通过 Su! 分享");
    if (*ppsz).is_null() { return -2147024882i32; }
    S_OK
}

unsafe extern "system" fn ec_get_icon(_this: *mut std::ffi::c_void, _psiia: *mut std::ffi::c_void, ppsz: *mut *mut u16) -> i32 {
    if ppsz.is_null() { return -2147024809i32; }
    if let Some(exe) = find_su_exe() {
        *ppsz = alloc_wide(&exe.to_string_lossy());
    } else {
        *ppsz = alloc_wide("");
    }
    S_OK
}

unsafe extern "system" fn ec_get_tooltip(_this: *mut std::ffi::c_void, _psiia: *mut std::ffi::c_void, ppsz: *mut *mut u16) -> i32 {
    if ppsz.is_null() { return -2147024809i32; }
    *ppsz = alloc_wide("通过 Su! 在局域网内分享文件");
    S_OK
}

unsafe extern "system" fn ec_get_canonical_name(_this: *mut std::ffi::c_void, guid: *mut GuidRaw) -> i32 {
    if guid.is_null() { return -2147024809i32; }
    *guid = GuidRaw { data1: CLSID_DATA1, data2: CLSID_DATA2, data3: CLSID_DATA3, data4: CLSID_DATA4 };
    S_OK
}

unsafe extern "system" fn ec_get_state(_this: *mut std::ffi::c_void, _psiia: *mut std::ffi::c_void, _ok: i32, pcmdstate: *mut u32) -> i32 {
    if pcmdstate.is_null() { return -2147024809i32; }
    *pcmdstate = 0; // ECS_ENABLED
    S_OK
}

unsafe extern "system" fn ec_invoke(_this: *mut std::ffi::c_void, psiia: *mut std::ffi::c_void, _pbc: *mut std::ffi::c_void) -> i32 {
    if let Some(exe) = find_su_exe() {
        let mut paths: Vec<String> = Vec::new();
        if !psiia.is_null() {
            let vtbl = *(psiia as *const *const IShellItemArrayVtbl);
            if !vtbl.is_null() {
                let mut count: u32 = 0;
                if ((*vtbl).get_count)(psiia, &mut count) == S_OK {
                    for i in 0..count {
                        let mut item: *mut std::ffi::c_void = std::ptr::null_mut();
                        if ((*vtbl).get_item_at)(psiia, i, &mut item) == S_OK && !item.is_null() {
                            let si_vtbl = *(item as *const *const IShellItemVtbl);
                            let mut dn: *mut u16 = std::ptr::null_mut();
                            if ((*si_vtbl).get_display_name)(item, SIGDN_FILESYSPATH, &mut dn) == S_OK && !dn.is_null() {
                                let mut len = 0usize;
                                while *dn.add(len) != 0 { len += 1; }
                                if let Ok(s) = String::from_utf16(std::slice::from_raw_parts(dn, len)) {
                                    paths.push(s);
                                }
                                CoTaskMemFree(dn as *mut std::ffi::c_void);
                            }
                            ((*si_vtbl).iunknown.release)(item);
                        }
                    }
                }
            }
        }
        if !paths.is_empty() {
            let tmp = std::env::temp_dir().join(format!("su_cli_{}.txt", std::process::id()));
            let _ = std::fs::write(&tmp, &paths.join("
"));
            let _ = std::process::Command::new(&exe).arg(tmp.to_string_lossy().to_string()).spawn();
        } else {
            let _ = std::process::Command::new(&exe).spawn();
        }
    }
    S_OK
}

unsafe extern "system" fn ec_get_flags(_this: *mut std::ffi::c_void, pdwflags: *mut u32) -> i32 {
    if pdwflags.is_null() { return -2147024809i32; }
    *pdwflags = 0;
    S_OK
}

unsafe extern "system" fn ec_enum_sub(_this: *mut std::ffi::c_void, ppenum: *mut *mut std::ffi::c_void) -> i32 {
    if !ppenum.is_null() { *ppenum = std::ptr::null_mut(); }
    S_OK
}

// ---- Exported COM functions ----

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GuidRaw, riid: *const GuidRaw, ppv: *mut *mut std::ffi::c_void) -> i32 {
    if rclsid.is_null() || ppv.is_null() { return -2147024809i32; }
    *ppv = std::ptr::null_mut();
    let r = &*rclsid;
    if r.data1 != CLSID_DATA1 || r.data2 != CLSID_DATA2 || r.data3 != CLSID_DATA3 || r.data4 != CLSID_DATA4 {
        return -2147221164i32; // CLASS_E_CLASSNOTAVAILABLE
    }
    qi(unsafe { &CLASS_FACTORY as *const ClassFactory as *mut std::ffi::c_void }, riid, ppv)
}

// Public function callable from commands.rs
pub fn register_shell_ext(register: bool) -> i32 {
    let dll_path = get_dll_path().to_string_lossy().to_string();
    if dll_path.is_empty() && register { return E_FAIL; }
    
    let clsid_str = format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_DATA1, CLSID_DATA2, CLSID_DATA3,
        CLSID_DATA4[0], CLSID_DATA4[1], CLSID_DATA4[2], CLSID_DATA4[3],
        CLSID_DATA4[4], CLSID_DATA4[5], CLSID_DATA4[6], CLSID_DATA4[7]
    );
    
    if register {
        // Clean up old non-COM entries first (from previous versions)
        regdelete("Software\\Classes\\*\\shell\\SuShare");
        regwrite(&[r"Software\Classes\CLSID\", &clsid_str].concat(), "", "Su! Shell Extension");
        regwrite(&[r"Software\Classes\CLSID\", &clsid_str, r"\InprocServer32"].concat(), "", &dll_path);
        regwrite(&[r"Software\Classes\CLSID\", &clsid_str, r"\InprocServer32"].concat(), "ThreadingModel", "Apartment");
        regwrite(r"Software\Classes\*\shellex\ContextMenuHandlers\SuShare", "", &clsid_str);
    } else {
        regdelete(r"Software\Classes\*\shellex\ContextMenuHandlers\SuShare");
        regdelete(&[r"Software\Classes\CLSID\", &clsid_str].concat());
        // Also restore old-style entry so user still has right-click menu
        // (Not restoring - user explicitly unregistered)
    }
    
    S_OK
}

pub fn regwrite(path: &str, name: &str, value: &str) {
    let k: Vec<u16> = std::ffi::OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    let n: Vec<u16> = std::ffi::OsStr::new(name).encode_wide().chain(std::iter::once(0)).collect();
    let v: Vec<u16> = std::ffi::OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let mut hkey: *mut std::ffi::c_void = std::ptr::null_mut();
        if RegCreateKeyExW(0x80000001usize as *mut std::ffi::c_void, k.as_ptr(), 0,
            std::ptr::null_mut(), 0, 0x00020006, std::ptr::null_mut(), &mut hkey, std::ptr::null_mut()) == 0 {
            RegSetValueExW(hkey, n.as_ptr(), 0, 1, v.as_ptr() as *const u8, (v.len() * 2) as u32);
            RegCloseKey(hkey);
        }
    }
}

pub fn regdelete(path: &str) {
    let p: Vec<u16> = std::ffi::OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    unsafe { RegDeleteTreeW(0x80000001usize as *mut std::ffi::c_void, p.as_ptr()); }
}extern "system" {
    fn RegCreateKeyExW(hKey: *mut std::ffi::c_void, lpSubKey: *const u16, Reserved: u32,
        lpClass: *const u16, dwOptions: u32, samDesired: u32,
        lpSecurityAttributes: *const std::ffi::c_void, phkResult: *mut *mut std::ffi::c_void,
        lpdwDisposition: *mut u32) -> i32;
    fn RegSetValueExW(hKey: *mut std::ffi::c_void, lpValueName: *const u16, Reserved: u32,
        dwType: u32, lpData: *const u8, cbData: u32) -> i32;
    fn RegCloseKey(hKey: *mut std::ffi::c_void) -> i32;
    fn RegDeleteTreeW(hKey: *mut std::ffi::c_void, lpSubKey: *const u16) -> i32;
}
