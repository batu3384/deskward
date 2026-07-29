//! C FFI for Flutter / Dart.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use deskward_core::auth::password::hash_password;
use deskward_core::connect::{connect_to_peer, resolve_peer};
use deskward_core::crypto::Identity;
use deskward_core::session_runtime::ClientSession;
use deskward_core::tailscale::system::fetch_status;
use deskward_core::tailscale::TailscaleStatus;
use serde::Serialize;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static SESSIONS: OnceLock<Mutex<HashMap<u64, ClientSession>>> = OnceLock::new();
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

fn sessions() -> &'static Mutex<HashMap<u64, ClientSession>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn to_json_c<T: Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(s) => CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

fn cstr(s: *const c_char) -> Option<String> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s) }.to_str().ok().map(|s| s.to_string())
}

#[derive(Serialize)]
struct PermissionsStatus {
    screen_recording: bool,
    accessibility: bool,
}

#[derive(Serialize)]
struct FrameJson {
    width: u32,
    height: u32,
    codec: String,
    data_b64: String,
}

#[no_mangle]
pub extern "C" fn deskward_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn deskward_tailscale_status() -> *mut c_char {
    match fetch_status() {
        Ok(status) => to_json_c(&status),
        Err(_) => {
            let fallback = TailscaleStatus::default();
            to_json_c(&fallback)
        }
    }
}

#[no_mangle]
pub extern "C" fn deskward_permissions_status() -> *mut c_char {
    let status = PermissionsStatus {
        screen_recording: platform::screen_recording_granted(),
        accessibility: platform::accessibility_granted(),
    };
    to_json_c(&status)
}

#[no_mangle]
pub extern "C" fn deskward_hash_password(plain: *const c_char) -> *mut c_char {
    let Some(plain) = cstr(plain) else {
        return std::ptr::null_mut();
    };
    match hash_password(&plain) {
        Ok(h) => CString::new(h.0).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns session id (>0) or negative error code.
#[no_mangle]
pub extern "C" fn deskward_connect(peer_name: *const c_char, password: *const c_char) -> i64 {
    let Some(peer_name) = cstr(peer_name) else {
        return -1;
    };
    let Some(password) = cstr(password) else {
        return -2;
    };

    let result = runtime().block_on(async {
        let status = fetch_status()?;
        let peer = resolve_peer(&status, &peer_name)?;
        if !peer.online {
            return Err(deskward_core::Error::PeerNotFound(peer_name.clone()));
        }
        let identity = Identity::generate();
        connect_to_peer(&peer.ipv4, "deskward-ctrl", &password, identity).await
    });

    match result {
        Ok(session) => {
            let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
            sessions().lock().unwrap().insert(id, session);
            id as i64
        }
        Err(deskward_core::Error::AuthFailed) => -3,
        Err(deskward_core::Error::PeerNotFound(_)) => -4,
        Err(deskward_core::Error::HandshakeFailed) => -5,
        Err(_) => -99,
    }
}

/// JSON frame or null if none.
#[no_mangle]
pub extern "C" fn deskward_session_poll_frame(session_id: u64) -> *mut c_char {
    let session = sessions().lock().unwrap().get(&session_id).cloned();
    let Some(session) = session else {
        return std::ptr::null_mut();
    };
    let frame = runtime().block_on(session.poll_frame());
    let Some(frame) = frame else {
        return std::ptr::null_mut();
    };
    use base64::Engine;
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(&frame.data);
    to_json_c(&FrameJson {
        width: frame.width,
        height: frame.height,
        codec: frame.codec,
        data_b64,
    })
}

#[no_mangle]
pub extern "C" fn deskward_session_pointer(
    session_id: u64,
    x: f64,
    y: f64,
    pressed: bool,
) -> i32 {
    let session = sessions().lock().unwrap().get(&session_id).cloned();
    let Some(session) = session else {
        return -1;
    };
    match runtime().block_on(session.send_pointer(x, y, pressed)) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn deskward_session_send_clipboard(
    session_id: u64,
    text: *const c_char,
) -> i32 {
    let Some(text) = cstr(text) else {
        return -1;
    };
    let session = sessions().lock().unwrap().get(&session_id).cloned();
    let Some(session) = session else {
        return -2;
    };
    match runtime().block_on(session.send_clipboard_text(&text)) {
        Ok(()) => 0,
        Err(_) => -3,
    }
}

/// JSON metrics `{ fps, frames_received, bytes_received, decoder }` or null.
#[no_mangle]
pub extern "C" fn deskward_session_metrics(session_id: u64) -> *mut c_char {
    let session = sessions().lock().unwrap().get(&session_id).cloned();
    let Some(session) = session else {
        return std::ptr::null_mut();
    };
    let metrics = runtime().block_on(session.metrics());
    to_json_c(&metrics)
}

#[no_mangle]
pub extern "C" fn deskward_session_disconnect(session_id: u64) -> i32 {
    let session = sessions().lock().unwrap().remove(&session_id);
    let Some(session) = session else {
        return -1;
    };
    match runtime().block_on(session.close()) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

mod platform {
    #[cfg(target_os = "macos")]
    #[link(name = "CoreGraphics", kind = "framework")]
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {}

    #[cfg(target_os = "macos")]
    pub fn screen_recording_granted() -> bool {
        extern "C" {
            fn CGPreflightScreenCaptureAccess() -> bool;
        }
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn screen_recording_granted() -> bool {
        false
    }

    #[cfg(target_os = "macos")]
    pub fn accessibility_granted() -> bool {
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn accessibility_granted() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_export() {
        let plain = CString::new("correct horse battery staple").unwrap();
        let hash = deskward_hash_password(plain.as_ptr());
        assert!(!hash.is_null());
        deskward_free_string(hash);
    }
}
