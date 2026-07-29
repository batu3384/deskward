//! macOS CGEvent input injection (Faz 1 — stub).

use deskward_core::input::{InputInjector, KeyEvent, PointerEvent};
use deskward_core::Result;
use tracing::debug;

pub struct MacInputInjector;

impl MacInputInjector {
    pub fn new() -> Self {
        Self
    }
}

impl InputInjector for MacInputInjector {
    fn move_pointer(&mut self, x: f64, y: f64) -> Result<()> {
        debug!(x, y, "MacInputInjector move (stub)");
        Ok(())
    }

    fn pointer_button(&mut self, ev: PointerEvent) -> Result<()> {
        debug!(?ev, "MacInputInjector button (stub)");
        Ok(())
    }

    fn key(&mut self, ev: KeyEvent) -> Result<()> {
        debug!(?ev, "MacInputInjector key (stub)");
        Ok(())
    }
}
