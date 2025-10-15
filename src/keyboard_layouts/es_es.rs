use crate::keyboard_layout::{KeyCombo, KeyboardLayout};

pub fn create_layout() -> KeyboardLayout {
    let mut layout = KeyboardLayout::new("es-ES", "Español");

    let key_trema = KeyCombo::new("Quote").with_shift();
    let key_acute = KeyCombo::new("Quote");
    let key_hat = KeyCombo::new("BracketRight").with_shift();
    let key_grave = KeyCombo::new("BracketRight");
    let key_tilde = KeyCombo::new("Digit4").with_alt_right();

    layout.chars.insert('A', KeyCombo::new("KeyA").with_shift());
    layout.chars.insert(
        'Ä',
        KeyCombo::new("KeyA")
            .with_shift()
            .with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'Á',
        KeyCombo::new("KeyA")
            .with_shift()
            .with_accent_key(key_acute.clone()),
    );
    layout.chars.insert(
        'Â',
        KeyCombo::new("KeyA")
            .with_shift()
            .with_accent_key(key_hat.clone()),
    );
    layout.chars.insert(
        'À',
        KeyCombo::new("KeyA")
            .with_shift()
            .with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'Ã',
        KeyCombo::new("KeyA")
            .with_shift()
            .with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('B', KeyCombo::new("KeyB").with_shift());
    layout.chars.insert('C', KeyCombo::new("KeyC").with_shift());
    layout.chars.insert('D', KeyCombo::new("KeyD").with_shift());
    layout.chars.insert('E', KeyCombo::new("KeyE").with_shift());
    layout.chars.insert(
        'Ë',
        KeyCombo::new("KeyE")
            .with_shift()
            .with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'É',
        KeyCombo::new("KeyE")
            .with_shift()
            .with_accent_key(key_acute.clone()),
    );
    layout.chars.insert(
        'Ê',
        KeyCombo::new("KeyE")
            .with_shift()
            .with_accent_key(key_hat.clone()),
    );
    layout.chars.insert(
        'È',
        KeyCombo::new("KeyE")
            .with_shift()
            .with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'Ẽ',
        KeyCombo::new("KeyE")
            .with_shift()
            .with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('F', KeyCombo::new("KeyF").with_shift());
    layout.chars.insert('G', KeyCombo::new("KeyG").with_shift());
    layout.chars.insert('H', KeyCombo::new("KeyH").with_shift());
    layout.chars.insert('I', KeyCombo::new("KeyI").with_shift());
    layout.chars.insert(
        'Ï',
        KeyCombo::new("KeyI")
            .with_shift()
            .with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'Í',
        KeyCombo::new("KeyI")
            .with_shift()
            .with_accent_key(key_acute.clone()),
    );
    layout.chars.insert(
        'Î',
        KeyCombo::new("KeyI")
            .with_shift()
            .with_accent_key(key_hat.clone()),
    );
    layout.chars.insert(
        'Ì',
        KeyCombo::new("KeyI")
            .with_shift()
            .with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'Ĩ',
        KeyCombo::new("KeyI")
            .with_shift()
            .with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('J', KeyCombo::new("KeyJ").with_shift());
    layout.chars.insert('K', KeyCombo::new("KeyK").with_shift());
    layout.chars.insert('L', KeyCombo::new("KeyL").with_shift());
    layout.chars.insert('M', KeyCombo::new("KeyM").with_shift());
    layout.chars.insert('N', KeyCombo::new("KeyN").with_shift());
    layout.chars.insert('O', KeyCombo::new("KeyO").with_shift());
    layout.chars.insert(
        'Ö',
        KeyCombo::new("KeyO")
            .with_shift()
            .with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'Ó',
        KeyCombo::new("KeyO")
            .with_shift()
            .with_accent_key(key_acute.clone()),
    );
    layout.chars.insert(
        'Ô',
        KeyCombo::new("KeyO")
            .with_shift()
            .with_accent_key(key_hat.clone()),
    );
    layout.chars.insert(
        'Ò',
        KeyCombo::new("KeyO")
            .with_shift()
            .with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'Õ',
        KeyCombo::new("KeyO")
            .with_shift()
            .with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('P', KeyCombo::new("KeyP").with_shift());
    layout.chars.insert('Q', KeyCombo::new("KeyQ").with_shift());
    layout.chars.insert('R', KeyCombo::new("KeyR").with_shift());
    layout.chars.insert('S', KeyCombo::new("KeyS").with_shift());
    layout.chars.insert('T', KeyCombo::new("KeyT").with_shift());
    layout.chars.insert('U', KeyCombo::new("KeyU").with_shift());
    layout.chars.insert(
        'Ü',
        KeyCombo::new("KeyU")
            .with_shift()
            .with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'Ú',
        KeyCombo::new("KeyU")
            .with_shift()
            .with_accent_key(key_acute.clone()),
    );
    layout.chars.insert(
        'Û',
        KeyCombo::new("KeyU")
            .with_shift()
            .with_accent_key(key_hat.clone()),
    );
    layout.chars.insert(
        'Ù',
        KeyCombo::new("KeyU")
            .with_shift()
            .with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'Ũ',
        KeyCombo::new("KeyU")
            .with_shift()
            .with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('V', KeyCombo::new("KeyV").with_shift());
    layout.chars.insert('W', KeyCombo::new("KeyW").with_shift());
    layout.chars.insert('X', KeyCombo::new("KeyX").with_shift());
    layout.chars.insert('Y', KeyCombo::new("KeyY").with_shift());
    layout.chars.insert('Z', KeyCombo::new("KeyZ").with_shift());

    layout.chars.insert('a', KeyCombo::new("KeyA"));
    layout.chars.insert(
        'ä',
        KeyCombo::new("KeyA").with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'á',
        KeyCombo::new("KeyA").with_accent_key(key_acute.clone()),
    );
    layout
        .chars
        .insert('â', KeyCombo::new("KeyA").with_accent_key(key_hat.clone()));
    layout.chars.insert(
        'à',
        KeyCombo::new("KeyA").with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'ã',
        KeyCombo::new("KeyA").with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('b', KeyCombo::new("KeyB"));
    layout.chars.insert('c', KeyCombo::new("KeyC"));
    layout.chars.insert('d', KeyCombo::new("KeyD"));
    layout.chars.insert('e', KeyCombo::new("KeyE"));
    layout.chars.insert(
        'ë',
        KeyCombo::new("KeyE").with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'é',
        KeyCombo::new("KeyE").with_accent_key(key_acute.clone()),
    );
    layout
        .chars
        .insert('ê', KeyCombo::new("KeyE").with_accent_key(key_hat.clone()));
    layout.chars.insert(
        'è',
        KeyCombo::new("KeyE").with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'ẽ',
        KeyCombo::new("KeyE").with_accent_key(key_tilde.clone()),
    );
    layout
        .chars
        .insert('€', KeyCombo::new("KeyE").with_alt_right());
    layout.chars.insert('f', KeyCombo::new("KeyF"));
    layout.chars.insert('g', KeyCombo::new("KeyG"));
    layout.chars.insert('h', KeyCombo::new("KeyH"));
    layout.chars.insert('i', KeyCombo::new("KeyI"));
    layout.chars.insert(
        'ï',
        KeyCombo::new("KeyI").with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'í',
        KeyCombo::new("KeyI").with_accent_key(key_acute.clone()),
    );
    layout
        .chars
        .insert('î', KeyCombo::new("KeyI").with_accent_key(key_hat.clone()));
    layout.chars.insert(
        'ì',
        KeyCombo::new("KeyI").with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'ĩ',
        KeyCombo::new("KeyI").with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('j', KeyCombo::new("KeyJ"));
    layout.chars.insert('k', KeyCombo::new("KeyK"));
    layout.chars.insert('l', KeyCombo::new("KeyL"));
    layout.chars.insert('m', KeyCombo::new("KeyM"));
    layout.chars.insert('n', KeyCombo::new("KeyN"));
    layout.chars.insert('o', KeyCombo::new("KeyO"));
    layout.chars.insert(
        'ö',
        KeyCombo::new("KeyO").with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'ó',
        KeyCombo::new("KeyO").with_accent_key(key_acute.clone()),
    );
    layout
        .chars
        .insert('ô', KeyCombo::new("KeyO").with_accent_key(key_hat.clone()));
    layout.chars.insert(
        'ò',
        KeyCombo::new("KeyO").with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'õ',
        KeyCombo::new("KeyO").with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('p', KeyCombo::new("KeyP"));
    layout.chars.insert('q', KeyCombo::new("KeyQ"));
    layout.chars.insert('r', KeyCombo::new("KeyR"));
    layout.chars.insert('s', KeyCombo::new("KeyS"));
    layout.chars.insert('t', KeyCombo::new("KeyT"));
    layout.chars.insert('u', KeyCombo::new("KeyU"));
    layout.chars.insert(
        'ü',
        KeyCombo::new("KeyU").with_accent_key(key_trema.clone()),
    );
    layout.chars.insert(
        'ú',
        KeyCombo::new("KeyU").with_accent_key(key_acute.clone()),
    );
    layout
        .chars
        .insert('û', KeyCombo::new("KeyU").with_accent_key(key_hat.clone()));
    layout.chars.insert(
        'ù',
        KeyCombo::new("KeyU").with_accent_key(key_grave.clone()),
    );
    layout.chars.insert(
        'ũ',
        KeyCombo::new("KeyU").with_accent_key(key_tilde.clone()),
    );
    layout.chars.insert('v', KeyCombo::new("KeyV"));
    layout.chars.insert('w', KeyCombo::new("KeyW"));
    layout.chars.insert('x', KeyCombo::new("KeyX"));
    layout.chars.insert('y', KeyCombo::new("KeyY"));
    layout.chars.insert('z', KeyCombo::new("KeyZ"));

    layout.chars.insert('º', KeyCombo::new("Backquote"));
    layout
        .chars
        .insert('ª', KeyCombo::new("Backquote").with_shift());
    layout
        .chars
        .insert('\\', KeyCombo::new("Backquote").with_alt_right());
    layout.chars.insert('1', KeyCombo::new("Digit1"));
    layout
        .chars
        .insert('!', KeyCombo::new("Digit1").with_shift());
    layout
        .chars
        .insert('|', KeyCombo::new("Digit1").with_alt_right());
    layout.chars.insert('2', KeyCombo::new("Digit2"));
    layout
        .chars
        .insert('"', KeyCombo::new("Digit2").with_shift());
    layout
        .chars
        .insert('@', KeyCombo::new("Digit2").with_alt_right());
    layout.chars.insert('3', KeyCombo::new("Digit3"));
    layout
        .chars
        .insert('·', KeyCombo::new("Digit3").with_shift());
    layout
        .chars
        .insert('#', KeyCombo::new("Digit3").with_alt_right());
    layout.chars.insert('4', KeyCombo::new("Digit4"));
    layout
        .chars
        .insert('$', KeyCombo::new("Digit4").with_shift());
    layout.chars.insert('5', KeyCombo::new("Digit5"));
    layout
        .chars
        .insert('%', KeyCombo::new("Digit5").with_shift());
    layout.chars.insert('6', KeyCombo::new("Digit6"));
    layout
        .chars
        .insert('&', KeyCombo::new("Digit6").with_shift());
    layout
        .chars
        .insert('¬', KeyCombo::new("Digit6").with_alt_right());
    layout.chars.insert('7', KeyCombo::new("Digit7"));
    layout
        .chars
        .insert('/', KeyCombo::new("Digit7").with_shift());
    layout.chars.insert('8', KeyCombo::new("Digit8"));
    layout
        .chars
        .insert('(', KeyCombo::new("Digit8").with_shift());
    layout.chars.insert('9', KeyCombo::new("Digit9"));
    layout
        .chars
        .insert(')', KeyCombo::new("Digit9").with_shift());
    layout.chars.insert('0', KeyCombo::new("Digit0"));
    layout
        .chars
        .insert('=', KeyCombo::new("Digit0").with_shift());
    layout.chars.insert('\'', KeyCombo::new("Minus"));
    layout
        .chars
        .insert('?', KeyCombo::new("Minus").with_shift());
    layout
        .chars
        .insert('¡', KeyCombo::new("Equal").with_dead_key());
    layout
        .chars
        .insert('¿', KeyCombo::new("Equal").with_shift());
    layout
        .chars
        .insert('[', KeyCombo::new("BracketLeft").with_alt_right());
    layout.chars.insert('+', KeyCombo::new("BracketRight"));
    layout
        .chars
        .insert('*', KeyCombo::new("BracketRight").with_shift());
    layout
        .chars
        .insert(']', KeyCombo::new("BracketRight").with_alt_right());
    layout.chars.insert('ñ', KeyCombo::new("Semicolon"));
    layout
        .chars
        .insert('Ñ', KeyCombo::new("Semicolon").with_shift());
    layout
        .chars
        .insert('{', KeyCombo::new("Quote").with_alt_right());
    layout.chars.insert('ç', KeyCombo::new("Backslash"));
    layout
        .chars
        .insert('Ç', KeyCombo::new("Backslash").with_shift());
    layout
        .chars
        .insert('}', KeyCombo::new("Backslash").with_alt_right());
    layout.chars.insert(',', KeyCombo::new("Comma"));
    layout
        .chars
        .insert(';', KeyCombo::new("Comma").with_shift());
    layout.chars.insert('.', KeyCombo::new("Period"));
    layout
        .chars
        .insert(':', KeyCombo::new("Period").with_shift());
    layout.chars.insert('-', KeyCombo::new("Slash"));
    layout
        .chars
        .insert('_', KeyCombo::new("Slash").with_shift());
    layout.chars.insert('<', KeyCombo::new("IntlBackslash"));
    layout
        .chars
        .insert('>', KeyCombo::new("IntlBackslash").with_shift());
    layout.chars.insert(' ', KeyCombo::new("Space"));
    layout.chars.insert('\n', KeyCombo::new("Enter"));

    layout
}
