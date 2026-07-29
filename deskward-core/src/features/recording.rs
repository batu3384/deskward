use crate::media::VideoFrame;

/// Optional session recording sink (Faz 3).
pub trait RecordingSink: Send {
    fn on_frame(&mut self, frame: &VideoFrame) -> crate::Result<()>;
    fn finalize(&mut self) -> crate::Result<()>;
}

pub struct NoopRecording;

impl RecordingSink for NoopRecording {
    fn on_frame(&mut self, _frame: &VideoFrame) -> crate::Result<()> {
        Ok(())
    }
    fn finalize(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
