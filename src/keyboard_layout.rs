use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeyCombo {
    pub key: String,
    pub shift: bool,
    pub alt_right: bool,
    pub dead_key: bool,
    pub accent_key: Option<Box<KeyCombo>>,
}

impl KeyCombo {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            shift: false,
            alt_right: false,
            dead_key: false,
            accent_key: None,
        }
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt_right(mut self) -> Self {
        self.alt_right = true;
        self
    }

    pub fn with_dead_key(mut self) -> Self {
        self.dead_key = true;
        self
    }

    pub fn with_accent_key(mut self, accent: KeyCombo) -> Self {
        self.accent_key = Some(Box::new(accent));
        self
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardLayout {
    pub iso_code: String,
    pub name: String,
    pub chars: HashMap<char, KeyCombo>,
}

impl KeyboardLayout {
    pub fn new(iso_code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            iso_code: iso_code.into(),
            name: name.into(),
            chars: HashMap::new(),
        }
    }

    pub fn with_char(mut self, c: char, combo: KeyCombo) -> Self {
        self.chars.insert(c, combo);
        self
    }

    pub fn get_char(&self, c: char) -> Option<&KeyCombo> {
        self.chars.get(&c)
    }
}

pub mod layouts {
    use super::*;
    use once_cell::sync::Lazy;

    pub static EN_US: Lazy<KeyboardLayout> =
        Lazy::new(crate::keyboard_layouts::en_us::create_layout);

    pub static ES_ES: Lazy<KeyboardLayout> =
        Lazy::new(crate::keyboard_layouts::es_es::create_layout);

    pub fn get_layout(iso_code: &str) -> Option<&'static KeyboardLayout> {
        match iso_code {
            "en-US" => Some(&EN_US),
            "es-ES" => Some(&ES_ES),
            _ => None,
        }
    }

    pub fn get_layout_or_default(iso_code: &str) -> &'static KeyboardLayout {
        get_layout(iso_code).unwrap_or(&EN_US)
    }
}
