//! Append-only session audit log (Faz 3).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

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
