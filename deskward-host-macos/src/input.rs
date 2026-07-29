//! macOS input injection via enigo (CGEvent backend).

use deskward_core::input::{InputInjector, KeyEvent, PointerEvent};
use deskward_core::Result;
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

pub struct MacInputInjector {
    enigo: Enigo,
}

impl MacInputInjector {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        Ok(Self { enigo })
    }
}

impl InputInjector for MacInputInjector {
    fn move_pointer(&mut self, x: f64, y: f64) -> Result<()> {
        self.enigo
            .move_mouse(x.round() as i32, y.round() as i32, Coordinate::Abs)
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))
    }

    fn pointer_button(&mut self, ev: PointerEvent) -> Result<()> {
        self.enigo
            .move_mouse(ev.x.round() as i32, ev.y.round() as i32, Coordinate::Abs)
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))?;
        let button = match ev.button {
            1 => Button::Right,
            2 => Button::Middle,
            _ => Button::Left,
        };
        let dir = if ev.pressed {
            Direction::Press
        } else {
            Direction::Release
        };
        self.enigo
            .button(button, dir)
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))
    }

    fn key(&mut self, ev: KeyEvent) -> Result<()> {
        let keycode = match ev {
            KeyEvent::Down { keycode } | KeyEvent::Up { keycode } => keycode,
        };
        let key = Key::Other(keycode);
        let dir = match ev {
            KeyEvent::Down { .. } => Direction::Press,
            KeyEvent::Up { .. } => Direction::Release,
        };
        self.enigo
            .key(key, dir)
            .map_err(|e| deskward_core::Error::Protocol(e.to_string()))
    }
}

// ponytail: CGEvent not Send; dedicated session task owns this exclusively
unsafe impl Send for MacInputInjector {}
