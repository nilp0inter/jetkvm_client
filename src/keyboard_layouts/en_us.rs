use crate::keyboard_layout::{KeyCombo, KeyboardLayout};

pub fn create_layout() -> KeyboardLayout {
    let mut layout = KeyboardLayout::new("en-US", "English (US)");

    layout.chars.insert('A', KeyCombo::new("KeyA").with_shift());
    layout.chars.insert('B', KeyCombo::new("KeyB").with_shift());
    layout.chars.insert('C', KeyCombo::new("KeyC").with_shift());
    layout.chars.insert('D', KeyCombo::new("KeyD").with_shift());
    layout.chars.insert('E', KeyCombo::new("KeyE").with_shift());
    layout.chars.insert('F', KeyCombo::new("KeyF").with_shift());
    layout.chars.insert('G', KeyCombo::new("KeyG").with_shift());
    layout.chars.insert('H', KeyCombo::new("KeyH").with_shift());
    layout.chars.insert('I', KeyCombo::new("KeyI").with_shift());
    layout.chars.insert('J', KeyCombo::new("KeyJ").with_shift());
    layout.chars.insert('K', KeyCombo::new("KeyK").with_shift());
    layout.chars.insert('L', KeyCombo::new("KeyL").with_shift());
    layout.chars.insert('M', KeyCombo::new("KeyM").with_shift());
    layout.chars.insert('N', KeyCombo::new("KeyN").with_shift());
    layout.chars.insert('O', KeyCombo::new("KeyO").with_shift());
    layout.chars.insert('P', KeyCombo::new("KeyP").with_shift());
    layout.chars.insert('Q', KeyCombo::new("KeyQ").with_shift());
    layout.chars.insert('R', KeyCombo::new("KeyR").with_shift());
    layout.chars.insert('S', KeyCombo::new("KeyS").with_shift());
    layout.chars.insert('T', KeyCombo::new("KeyT").with_shift());
    layout.chars.insert('U', KeyCombo::new("KeyU").with_shift());
    layout.chars.insert('V', KeyCombo::new("KeyV").with_shift());
    layout.chars.insert('W', KeyCombo::new("KeyW").with_shift());
    layout.chars.insert('X', KeyCombo::new("KeyX").with_shift());
    layout.chars.insert('Y', KeyCombo::new("KeyY").with_shift());
    layout.chars.insert('Z', KeyCombo::new("KeyZ").with_shift());

    layout.chars.insert('a', KeyCombo::new("KeyA"));
    layout.chars.insert('b', KeyCombo::new("KeyB"));
    layout.chars.insert('c', KeyCombo::new("KeyC"));
    layout.chars.insert('d', KeyCombo::new("KeyD"));
    layout.chars.insert('e', KeyCombo::new("KeyE"));
    layout.chars.insert('f', KeyCombo::new("KeyF"));
    layout.chars.insert('g', KeyCombo::new("KeyG"));
    layout.chars.insert('h', KeyCombo::new("KeyH"));
    layout.chars.insert('i', KeyCombo::new("KeyI"));
    layout.chars.insert('j', KeyCombo::new("KeyJ"));
    layout.chars.insert('k', KeyCombo::new("KeyK"));
    layout.chars.insert('l', KeyCombo::new("KeyL"));
    layout.chars.insert('m', KeyCombo::new("KeyM"));
    layout.chars.insert('n', KeyCombo::new("KeyN"));
    layout.chars.insert('o', KeyCombo::new("KeyO"));
    layout.chars.insert('p', KeyCombo::new("KeyP"));
    layout.chars.insert('q', KeyCombo::new("KeyQ"));
    layout.chars.insert('r', KeyCombo::new("KeyR"));
    layout.chars.insert('s', KeyCombo::new("KeyS"));
    layout.chars.insert('t', KeyCombo::new("KeyT"));
    layout.chars.insert('u', KeyCombo::new("KeyU"));
    layout.chars.insert('v', KeyCombo::new("KeyV"));
    layout.chars.insert('w', KeyCombo::new("KeyW"));
    layout.chars.insert('x', KeyCombo::new("KeyX"));
    layout.chars.insert('y', KeyCombo::new("KeyY"));
    layout.chars.insert('z', KeyCombo::new("KeyZ"));

    layout.chars.insert('1', KeyCombo::new("Digit1"));
    layout
        .chars
        .insert('!', KeyCombo::new("Digit1").with_shift());
    layout.chars.insert('2', KeyCombo::new("Digit2"));
    layout
        .chars
        .insert('@', KeyCombo::new("Digit2").with_shift());
    layout.chars.insert('3', KeyCombo::new("Digit3"));
    layout
        .chars
        .insert('#', KeyCombo::new("Digit3").with_shift());
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
        .insert('^', KeyCombo::new("Digit6").with_shift());
    layout.chars.insert('7', KeyCombo::new("Digit7"));
    layout
        .chars
        .insert('&', KeyCombo::new("Digit7").with_shift());
    layout.chars.insert('8', KeyCombo::new("Digit8"));
    layout
        .chars
        .insert('*', KeyCombo::new("Digit8").with_shift());
    layout.chars.insert('9', KeyCombo::new("Digit9"));
    layout
        .chars
        .insert('(', KeyCombo::new("Digit9").with_shift());
    layout.chars.insert('0', KeyCombo::new("Digit0"));
    layout
        .chars
        .insert(')', KeyCombo::new("Digit0").with_shift());

    layout.chars.insert('-', KeyCombo::new("Minus"));
    layout
        .chars
        .insert('_', KeyCombo::new("Minus").with_shift());
    layout.chars.insert('=', KeyCombo::new("Equal"));
    layout
        .chars
        .insert('+', KeyCombo::new("Equal").with_shift());
    layout.chars.insert('\'', KeyCombo::new("Quote"));
    layout
        .chars
        .insert('"', KeyCombo::new("Quote").with_shift());
    layout.chars.insert(',', KeyCombo::new("Comma"));
    layout
        .chars
        .insert('<', KeyCombo::new("Comma").with_shift());
    layout.chars.insert('/', KeyCombo::new("Slash"));
    layout
        .chars
        .insert('?', KeyCombo::new("Slash").with_shift());
    layout.chars.insert('.', KeyCombo::new("Period"));
    layout
        .chars
        .insert('>', KeyCombo::new("Period").with_shift());
    layout.chars.insert(';', KeyCombo::new("Semicolon"));
    layout
        .chars
        .insert(':', KeyCombo::new("Semicolon").with_shift());
    layout.chars.insert('[', KeyCombo::new("BracketLeft"));
    layout
        .chars
        .insert('{', KeyCombo::new("BracketLeft").with_shift());
    layout.chars.insert(']', KeyCombo::new("BracketRight"));
    layout
        .chars
        .insert('}', KeyCombo::new("BracketRight").with_shift());
    layout.chars.insert('\\', KeyCombo::new("Backslash"));
    layout
        .chars
        .insert('|', KeyCombo::new("Backslash").with_shift());
    layout.chars.insert('`', KeyCombo::new("Backquote"));
    layout
        .chars
        .insert('~', KeyCombo::new("Backquote").with_shift());
    layout.chars.insert(' ', KeyCombo::new("Space"));
    layout.chars.insert('\n', KeyCombo::new("Enter"));

    layout
}
