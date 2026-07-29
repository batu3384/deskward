//! Reads ~/.deskward/setup.json — bridge until FFI lands.

use std::fs;
use std::path::PathBuf;

use deskward_core::auth::password::StoredPasswordHash;
use deskward_core::checklist::SetupSnapshot;
use deskward_core::host_gate::may_listen_as_host;

#[derive(serde::Deserialize)]
pub struct SetupFile {
    pub snapshot: SetupSnapshot,
    pub user_armed: bool,
    pub password_hash: Option<String>,
    pub codec: Option<String>,
}

fn setup_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".deskward")
        .join("setup.json")
}

pub fn load_setup() -> std::io::Result<Option<SetupFile>> {
    let path = setup_path();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let file: SetupFile = serde_json::from_str(&raw).map_err(std::io::Error::other)?;
    Ok(Some(file))
}

pub fn setup_gate_open() -> std::io::Result<bool> {
    if std::env::var("DESKWARD_FORCE_LISTEN").as_deref() == Ok("1") {
        return Ok(true);
    }
    let Some(file) = load_setup()? else {
        return Ok(false);
    };
    Ok(may_listen_as_host(&file.snapshot, file.user_armed))
}

pub fn password_hash_from_setup() -> Option<StoredPasswordHash> {
    load_setup()
        .ok()
        .flatten()
        .and_then(|f| f.password_hash.map(StoredPasswordHash))
}

pub fn codec_profile_from_setup() -> deskward_core::features::frame_codec::CodecProfile {
    load_setup()
        .ok()
        .flatten()
        .and_then(|f| f.codec)
        .map(|c| deskward_core::features::frame_codec::CodecProfile::from_wire(&c))
        .unwrap_or_else(deskward_core::features::frame_codec::CodecProfile::from_env)
}
