//! Clipboard apply roundtrip (no OS clipboard in CI).

use deskward_core::features::clipboard::{ClipboardPayload, ClipboardSync};
use deskward_core::Result;

struct MockClipboard {
    last: Option<String>,
}

impl ClipboardSync for MockClipboard {
    fn push_local(&mut self) -> Result<Option<ClipboardPayload>> {
        Ok(self.last.as_ref().map(|t| ClipboardPayload {
            mime: "text/plain".into(),
            data: t.clone().into_bytes(),
        }))
    }

    fn apply_remote(&mut self, payload: &ClipboardPayload) -> Result<()> {
        self.last = Some(String::from_utf8_lossy(&payload.data).into_owned());
        Ok(())
    }
}

#[test]
fn clipboard_text_roundtrip() {
    let mut cb = MockClipboard { last: None };
    cb.apply_remote(&ClipboardPayload {
        mime: "text/plain".into(),
        data: b"merhaba deskward".to_vec(),
    })
    .unwrap();
    let out = cb.push_local().unwrap().unwrap();
    assert_eq!(String::from_utf8(out.data).unwrap(), "merhaba deskward");
}
