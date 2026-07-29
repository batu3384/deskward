//! Append-only session audit log (JSONL).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::Result;

#[derive(Serialize)]
pub struct AuditEvent<'a> {
    pub ts: u64,
    pub event: &'a str,
    pub peer_id: &'a str,
    pub detail: &'a str,
}

pub fn append(path: &Path, event: AuditEvent<'_>) -> std::io::Result<()> {
    let line = serde_json::to_string(&event).map_err(std::io::Error::other)?;
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{line}")?;
    Ok(())
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Log when `DESKWARD_AUDIT_LOG` points to a writable path.
pub fn log_session(peer_id: &str, event: &str, detail: &str) -> Result<()> {
    let Ok(path) = std::env::var("DESKWARD_AUDIT_LOG") else {
        return Ok(());
    };
    append(
        Path::new(&path),
        AuditEvent {
            ts: now_ts(),
            event,
            peer_id,
            detail,
        },
    )
    .map_err(crate::Error::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn append_writes_jsonl() {
        let dir = std::env::temp_dir().join(format!("deskward-audit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");
        append(
            &path,
            AuditEvent {
                ts: 1,
                event: "session_connect",
                peer_id: "mac-host",
                detail: "ok",
            },
        )
        .unwrap();
        let mut s = String::new();
        std::fs::File::open(&path)
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert!(s.contains("session_connect"));
        assert!(s.contains("mac-host"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
