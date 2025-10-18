//! winit → HID translation: maintains keyboard and mouse state, emits the
//! HID reports that the JetKVM RPCs (`keyboardReport`, `absMouseReport`,
//! `wheelReport`) expect.

use arrayvec::ArrayVec;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard { modifier: u8, keys: Vec<u8> },
    AbsMouse { x: i64, y: i64, buttons: u8 },
    Wheel { dy: i64 },
}

/// Tracks the current set of held keys + active modifier byte.
/// Mirrors a real boot-protocol USB HID keyboard report (1 modifier byte +
/// up to 6 keys).
pub struct KeyboardState {
    pub modifier: u8,
    pub keys: ArrayVec<u8, 6>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            modifier: 0,
            keys: ArrayVec::new(),
        }
    }
}

impl KeyboardState {
    /// Returns Some(InputEvent::Keyboard) if the report has changed.
    pub fn handle(&mut self, code: KeyCode, state: ElementState) -> Option<InputEvent> {
        let pressed = state == ElementState::Pressed;
        if let Some(mask) = modifier_mask(code) {
            let new_mod = if pressed {
                self.modifier | mask
            } else {
                self.modifier & !mask
            };
            if new_mod == self.modifier {
                return None;
            }
            self.modifier = new_mod;
            return Some(self.emit());
        }

        let Some(hid) = keycode_to_hid(code) else {
            return None;
        };
        let already = self.keys.contains(&hid);
        let changed = if pressed {
            if !already && self.keys.try_push(hid).is_ok() {
                true
            } else {
                false
            }
        } else if let Some(idx) = self.keys.iter().position(|&k| k == hid) {
            self.keys.remove(idx);
            true
        } else {
            false
        };

        if changed {
            Some(self.emit())
        } else {
            None
        }
    }

    fn emit(&self) -> InputEvent {
        InputEvent::Keyboard {
            modifier: self.modifier,
            keys: self.keys.iter().copied().collect(),
        }
    }
}

#[derive(Default)]
pub struct MouseState {
    pub buttons: u8,
    pub last_x: i64,
    pub last_y: i64,
}

impl MouseState {
    pub fn handle_motion(
        &mut self,
        widget_x: f64,
        widget_y: f64,
        widget_w: f64,
        widget_h: f64,
        remote_w: u32,
        remote_h: u32,
    ) -> InputEvent {
        let (rx, ry) =
            map_to_remote(widget_x, widget_y, widget_w, widget_h, remote_w, remote_h);
        self.last_x = rx;
        self.last_y = ry;
        InputEvent::AbsMouse {
            x: rx,
            y: ry,
            buttons: self.buttons,
        }
    }

    pub fn handle_button(&mut self, button: MouseButton, state: ElementState) -> InputEvent {
        let mask = match button {
            MouseButton::Left => 0b001,
            MouseButton::Right => 0b010,
            MouseButton::Middle => 0b100,
            _ => 0,
        };
        if state == ElementState::Pressed {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }
        InputEvent::AbsMouse {
            x: self.last_x,
            y: self.last_y,
            buttons: self.buttons,
        }
    }

    pub fn handle_wheel(&self, delta: MouseScrollDelta) -> Option<InputEvent> {
        let dy = match delta {
            MouseScrollDelta::LineDelta(_, y) => y as f64,
            MouseScrollDelta::PixelDelta(p) => p.y / 30.0,
        };
        if dy.abs() < 0.01 {
            return None;
        }
        // JetKVM wheel report convention: positive = scroll up.
        let v = dy.round() as i64;
        if v == 0 {
            None
        } else {
            Some(InputEvent::Wheel { dy: v })
        }
    }
}

fn map_to_remote(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    remote_w: u32,
    remote_h: u32,
) -> (i64, i64) {
    if w <= 0.0 || h <= 0.0 {
        return (0, 0);
    }
    let rx = ((x.max(0.0).min(w) / w) * remote_w as f64) as i64;
    let ry = ((y.max(0.0).min(h) / h) * remote_h as f64) as i64;
    (rx.clamp(0, remote_w as i64 - 1), ry.clamp(0, remote_h as i64 - 1))
}

fn modifier_mask(code: KeyCode) -> Option<u8> {
    Some(match code {
        KeyCode::ControlLeft => 0x01,
        KeyCode::ShiftLeft => 0x02,
        KeyCode::AltLeft => 0x04,
        KeyCode::SuperLeft => 0x08,
        KeyCode::ControlRight => 0x10,
        KeyCode::ShiftRight => 0x20,
        KeyCode::AltRight => 0x40,
        KeyCode::SuperRight => 0x80,
        _ => return None,
    })
}

/// Map winit's physical `KeyCode` to USB HID Usage Page 0x07 keyboard codes.
pub fn keycode_to_hid(code: KeyCode) -> Option<u8> {
    Some(match code {
        // Letters
        KeyCode::KeyA => 0x04,
        KeyCode::KeyB => 0x05,
        KeyCode::KeyC => 0x06,
        KeyCode::KeyD => 0x07,
        KeyCode::KeyE => 0x08,
        KeyCode::KeyF => 0x09,
        KeyCode::KeyG => 0x0A,
        KeyCode::KeyH => 0x0B,
        KeyCode::KeyI => 0x0C,
        KeyCode::KeyJ => 0x0D,
        KeyCode::KeyK => 0x0E,
        KeyCode::KeyL => 0x0F,
        KeyCode::KeyM => 0x10,
        KeyCode::KeyN => 0x11,
        KeyCode::KeyO => 0x12,
        KeyCode::KeyP => 0x13,
        KeyCode::KeyQ => 0x14,
        KeyCode::KeyR => 0x15,
        KeyCode::KeyS => 0x16,
        KeyCode::KeyT => 0x17,
        KeyCode::KeyU => 0x18,
        KeyCode::KeyV => 0x19,
        KeyCode::KeyW => 0x1A,
        KeyCode::KeyX => 0x1B,
        KeyCode::KeyY => 0x1C,
        KeyCode::KeyZ => 0x1D,
        // Digits
        KeyCode::Digit1 => 0x1E,
        KeyCode::Digit2 => 0x1F,
        KeyCode::Digit3 => 0x20,
        KeyCode::Digit4 => 0x21,
        KeyCode::Digit5 => 0x22,
        KeyCode::Digit6 => 0x23,
        KeyCode::Digit7 => 0x24,
        KeyCode::Digit8 => 0x25,
        KeyCode::Digit9 => 0x26,
        KeyCode::Digit0 => 0x27,
        // Whitespace / control
        KeyCode::Enter => 0x28,
        KeyCode::Escape => 0x29,
        KeyCode::Backspace => 0x2A,
        KeyCode::Tab => 0x2B,
        KeyCode::Space => 0x2C,
        // Symbols
        KeyCode::Minus => 0x2D,
        KeyCode::Equal => 0x2E,
        KeyCode::BracketLeft => 0x2F,
        KeyCode::BracketRight => 0x30,
        KeyCode::Backslash => 0x31,
        KeyCode::Semicolon => 0x33,
        KeyCode::Quote => 0x34,
        KeyCode::Backquote => 0x35,
        KeyCode::Comma => 0x36,
        KeyCode::Period => 0x37,
        KeyCode::Slash => 0x38,
        KeyCode::CapsLock => 0x39,
        // Function keys
        KeyCode::F1 => 0x3A,
        KeyCode::F2 => 0x3B,
        KeyCode::F3 => 0x3C,
        KeyCode::F4 => 0x3D,
        KeyCode::F5 => 0x3E,
        KeyCode::F6 => 0x3F,
        KeyCode::F7 => 0x40,
        KeyCode::F8 => 0x41,
        KeyCode::F9 => 0x42,
        KeyCode::F10 => 0x43,
        KeyCode::F11 => 0x44,
        KeyCode::F12 => 0x45,
        KeyCode::PrintScreen => 0x46,
        KeyCode::ScrollLock => 0x47,
        KeyCode::Pause => 0x48,
        // Navigation
        KeyCode::Insert => 0x49,
        KeyCode::Home => 0x4A,
        KeyCode::PageUp => 0x4B,
        KeyCode::Delete => 0x4C,
        KeyCode::End => 0x4D,
        KeyCode::PageDown => 0x4E,
        KeyCode::ArrowRight => 0x4F,
        KeyCode::ArrowLeft => 0x50,
        KeyCode::ArrowDown => 0x51,
        KeyCode::ArrowUp => 0x52,
        // Numpad
        KeyCode::NumLock => 0x53,
        KeyCode::NumpadDivide => 0x54,
        KeyCode::NumpadMultiply => 0x55,
        KeyCode::NumpadSubtract => 0x56,
        KeyCode::NumpadAdd => 0x57,
        KeyCode::NumpadEnter => 0x58,
        KeyCode::Numpad1 => 0x59,
        KeyCode::Numpad2 => 0x5A,
        KeyCode::Numpad3 => 0x5B,
        KeyCode::Numpad4 => 0x5C,
        KeyCode::Numpad5 => 0x5D,
        KeyCode::Numpad6 => 0x5E,
        KeyCode::Numpad7 => 0x5F,
        KeyCode::Numpad8 => 0x60,
        KeyCode::Numpad9 => 0x61,
        KeyCode::Numpad0 => 0x62,
        KeyCode::NumpadDecimal => 0x63,
        KeyCode::IntlBackslash => 0x64,
        KeyCode::ContextMenu => 0x65,
        // International
        KeyCode::IntlRo => 0x87,
        KeyCode::KanaMode => 0x88,
        KeyCode::IntlYen => 0x89,
        KeyCode::Convert => 0x8A,
        KeyCode::NonConvert => 0x8B,
        _ => return None,
    })
}
