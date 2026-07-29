//! Remote input injection traits (Faz 1+ implementations per platform).

use crate::Result;

#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    pub x: f64,
    pub y: f64,
    pub button: u8,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Down { keycode: u32 },
    Up { keycode: u32 },
}

/// Inject mouse/keyboard into host OS.
pub trait InputInjector: Send {
    fn move_pointer(&mut self, x: f64, y: f64) -> Result<()>;
    fn pointer_button(&mut self, ev: PointerEvent) -> Result<()>;
    fn key(&mut self, ev: KeyEvent) -> Result<()>;
}

/// Map controller touch/gesture to remote pointer (iOS).
pub trait RemoteInputMapper: Send {
    fn touch_down(&mut self, x: f64, y: f64) -> Result<()>;
    fn touch_move(&mut self, x: f64, y: f64) -> Result<()>;
    fn touch_up(&mut self, x: f64, y: f64) -> Result<()>;
}
