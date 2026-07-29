//! Windows SendInput stub (Faz 2).

use deskward_core::input::{InputInjector, KeyEvent, PointerEvent};
use deskward_core::Result;
use tracing::debug;

pub struct WinInputInjector;

impl InputInjector for WinInputInjector {
    fn move_pointer(&mut self, x: f64, y: f64) -> Result<()> {
        debug!(x, y, "WinInputInjector move (stub)");
        Ok(())
    }

    fn pointer_button(&mut self, ev: PointerEvent) -> Result<()> {
        debug!(?ev, "WinInputInjector button (stub)");
        Ok(())
    }

    fn key(&mut self, ev: KeyEvent) -> Result<()> {
        debug!(?ev, "WinInputInjector key (stub)");
        Ok(())
    }
}
