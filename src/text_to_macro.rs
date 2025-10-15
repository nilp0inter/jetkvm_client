use crate::keyboard_layout::{layouts, KeyboardLayout};
use crate::keyboard_mappings::{key_name_to_hid, modifier_name_to_mask};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct MacroStep {
    pub keys: Vec<u8>,
    pub modifier: u8,
    pub delay_ms: u64,
}

impl MacroStep {
    pub fn new(keys: Vec<u8>, modifier: u8, delay_ms: u64) -> Self {
        Self {
            keys,
            modifier,
            delay_ms,
        }
    }
}

pub fn text_to_macro_steps(
    text: &str,
    layout: &KeyboardLayout,
    delay_ms: u64,
) -> Result<Vec<MacroStep>> {
    let mut steps = Vec::new();

    for c in text.chars() {
        let key_combo = layout
            .get_char(c)
            .ok_or_else(|| anyhow!("Character '{}' not found in layout {}", c, layout.iso_code))?;

        if let Some(accent_key) = &key_combo.accent_key {
            let accent_hid = key_name_to_hid(&accent_key.key)
                .ok_or_else(|| anyhow!("Invalid accent key: {}", accent_key.key))?;

            let mut accent_modifier = 0u8;
            if accent_key.shift {
                accent_modifier |= modifier_name_to_mask("ShiftLeft").unwrap_or(0);
            }
            if accent_key.alt_right {
                accent_modifier |= modifier_name_to_mask("AltRight").unwrap_or(0);
            }

            steps.push(MacroStep::new(vec![accent_hid], accent_modifier, delay_ms));
        }

        let key_hid = key_name_to_hid(&key_combo.key)
            .ok_or_else(|| anyhow!("Invalid key: {}", key_combo.key))?;

        let mut modifier = 0u8;
        if key_combo.shift {
            modifier |= modifier_name_to_mask("ShiftLeft").unwrap_or(0);
        }
        if key_combo.alt_right {
            modifier |= modifier_name_to_mask("AltRight").unwrap_or(0);
        }

        steps.push(MacroStep::new(vec![key_hid], modifier, delay_ms));

        if key_combo.dead_key {
            let space_hid =
                key_name_to_hid("Space").ok_or_else(|| anyhow!("Space key not found"))?;
            steps.push(MacroStep::new(vec![space_hid], 0, delay_ms));
        }
    }

    Ok(steps)
}

pub fn text_to_macro_steps_with_layout_code(
    text: &str,
    iso_code: &str,
    delay_ms: u64,
) -> Result<Vec<MacroStep>> {
    let layout = layouts::get_layout_or_default(iso_code);
    text_to_macro_steps(text, layout, delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_en_us_layout_basic_chars() {
        let layout = layouts::get_layout("en-US").unwrap();

        assert!(layout.get_char('a').is_some());
        assert!(layout.get_char('A').is_some());
        assert!(layout.get_char('1').is_some());
        assert!(layout.get_char('!').is_some());
        assert!(layout.get_char(' ').is_some());
        assert!(layout.get_char('\n').is_some());
    }

    #[test]
    fn test_en_us_uppercase_requires_shift() {
        let layout = layouts::get_layout("en-US").unwrap();

        let lowercase_a = layout.get_char('a').unwrap();
        let uppercase_a = layout.get_char('A').unwrap();

        assert!(!lowercase_a.shift);
        assert!(uppercase_a.shift);
    }

    #[test]
    fn test_es_es_layout_accented_chars() {
        let layout = layouts::get_layout("es-ES").unwrap();

        assert!(layout.get_char('á').is_some());
        assert!(layout.get_char('é').is_some());
        assert!(layout.get_char('í').is_some());
        assert!(layout.get_char('ó').is_some());
        assert!(layout.get_char('ú').is_some());
        assert!(layout.get_char('ñ').is_some());
        assert!(layout.get_char('Ñ').is_some());
    }

    #[test]
    fn test_es_es_accented_chars_have_accent_key() {
        let layout = layouts::get_layout("es-ES").unwrap();

        let a_acute = layout.get_char('á').unwrap();
        assert!(a_acute.accent_key.is_some());

        let plain_a = layout.get_char('a').unwrap();
        assert!(plain_a.accent_key.is_none());
    }

    #[test]
    fn test_text_to_macro_simple() {
        let result = text_to_macro_steps_with_layout_code("hello", "en-US", 20);
        assert!(result.is_ok());

        let steps = result.unwrap();
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn test_text_to_macro_with_accents() {
        let result = text_to_macro_steps_with_layout_code("hola", "es-ES", 20);
        assert!(result.is_ok());

        let steps = result.unwrap();
        assert_eq!(steps.len(), 4);
    }

    #[test]
    fn test_text_to_macro_with_accented_char() {
        let result = text_to_macro_steps_with_layout_code("á", "es-ES", 20);
        assert!(result.is_ok());

        let steps = result.unwrap();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn test_unsupported_char_returns_error() {
        let result = text_to_macro_steps_with_layout_code("日本", "en-US", 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_layout_fallback() {
        let layout = layouts::get_layout_or_default("invalid-code");
        assert_eq!(layout.iso_code, "en-US");
    }

    #[test]
    fn test_key_mappings_exist() {
        assert_eq!(key_name_to_hid("KeyA"), Some(0x04));
        assert_eq!(key_name_to_hid("Enter"), Some(0x28));
        assert_eq!(key_name_to_hid("Space"), Some(0x2c));

        assert_eq!(modifier_name_to_mask("ShiftLeft"), Some(0x02));
        assert_eq!(modifier_name_to_mask("ControlLeft"), Some(0x01));
        assert_eq!(modifier_name_to_mask("AltRight"), Some(0x40));
    }
}
