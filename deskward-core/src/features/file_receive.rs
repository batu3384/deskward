//! Assemble incoming file chunks on host.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::features::file_transfer::FileChunk;
use crate::{Error, Result};

pub struct FileReceiver {
    dir: PathBuf,
    open: HashMap<String, File>,
    names: HashMap<String, String>,
}

impl FileReceiver {
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            open: HashMap::new(),
            names: HashMap::new(),
        })
    }

    pub fn on_offer(&mut self, path: &str, _size: u64, session_id: &str) -> Result<()> {
        let name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.bin");
        let dest = self.dir.join(format!("{session_id}-{name}"));
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dest)?;
        self.open.insert(session_id.to_string(), file);
        self.names
            .insert(session_id.to_string(), dest.to_string_lossy().into());
        Ok(())
    }

    pub fn on_chunk(&mut self, chunk: &FileChunk) -> Result<()> {
        let file = self
            .open
            .get_mut(&chunk.session_id)
            .ok_or_else(|| Error::Protocol("unknown file session".into()))?;
        file.seek(SeekFrom::Start(chunk.offset))?;
        file.write_all(&chunk.data)?;
        if chunk.final_chunk {
            file.flush()?;
        }
        Ok(())
    }

    pub fn on_complete(&mut self, session_id: &str) -> Result<Option<PathBuf>> {
        if let Some(mut file) = self.open.remove(session_id) {
            file.flush()?;
        }
        Ok(self.names.remove(session_id).map(PathBuf::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_chunked_file() {
        let dir = std::env::temp_dir().join("deskward-test-files");
        let _ = fs::remove_dir_all(&dir);
        let mut rx = FileReceiver::new(&dir).unwrap();
        rx.on_offer("doc.txt", 11, "s1").unwrap();
        rx.on_chunk(&FileChunk {
            session_id: "s1".into(),
            offset: 0,
            data: b"hello world".to_vec(),
            final_chunk: true,
        })
        .unwrap();
        let path = rx.on_complete("s1").unwrap().unwrap();
        let text = fs::read_to_string(path).unwrap();
        assert_eq!(text, "hello world");
        let _ = fs::remove_dir_all(dir);
    }
}
