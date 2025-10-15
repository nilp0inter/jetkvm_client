# Keyboard Layout Support

This module provides comprehensive keyboard layout support for the JetKVM client, enabling you to send text with proper character mapping for different languages and keyboard layouts.

## Features

- **Multiple Keyboard Layouts**: Support for US English and Spanish layouts (with more to come)
- **Dead Key Support**: Proper handling of accent marks and diacritics (á, é, í, ó, ú, ñ, etc.)
- **HID Key Mapping**: Complete USB HID usage table implementation
- **Layout-Aware Text Conversion**: Automatically converts text to proper key combinations

## Architecture

### Modules

- **`keyboard_mappings`**: HID key codes and modifier masks
- **`keyboard_layout`**: Layout type definitions and management
- **`keyboard_layouts/`**: Individual layout implementations
  - `en_us`: English (US) layout
  - `es_es`: Spanish (Spain) layout with dead keys for accents
- **`text_to_macro`**: Text-to-macro conversion logic

## Usage

### Basic Example

```rust
use jetkvm_client::keyboard::send_text_with_layout;
use jetkvm_client::jetkvm_rpc_client::JetKvmRpcClient;

// Send text with US English layout
send_text_with_layout(&client, "Hello World!\n", "en-US", 20).await?;

// Send text with Spanish layout (with accents)
send_text_with_layout(&client, "¡Hola! ¿Cómo estás?\n", "es-ES", 20).await?;
```

### Command-Line Example

```bash
# Using the example program
cargo run --example send_text_with_layout -- \
    -H 192.168.1.100 \
    -P your_password \
    -l es-ES
```

### How It Works

1. **Character Lookup**: Each character is looked up in the layout's character map
2. **Dead Key Handling**: If a character requires an accent, the accent key is sent first
3. **Key Combination**: The main key is sent with appropriate modifiers (Shift, AltGr)
4. **Dead Key Completion**: If needed, a Space is sent to complete the dead key sequence

### Example: Spanish "á"

For the character "á" in Spanish layout:
1. Send `Quote` key (acute accent - dead key)
2. Send `KeyA` key
3. Result: "á" appears on screen

## Supported Layouts

### en-US (English - United States)
- All ASCII characters
- Standard US keyboard layout
- No dead keys

### es-ES (Spanish - Spain)
- All Spanish characters including ñ, Ñ
- Accented vowels: á, é, í, ó, ú (and uppercase)
- Special characters: ¡, ¿, º, ª, €
- Dead keys for: acute (´), grave (`), circumflex (^), tilde (~), umlaut (¨)

## Adding New Layouts

To add a new keyboard layout:

1. Create a new file in `src/keyboard_layouts/` (e.g., `fr_fr.rs`)
2. Implement the `create_layout()` function:

```rust
use crate::keyboard_layout::{KeyCombo, KeyboardLayout};

pub fn create_layout() -> KeyboardLayout {
    let mut layout = KeyboardLayout::new("fr-FR", "Français");
    
    // Define character mappings
    layout.chars.insert('a', KeyCombo::new("KeyA"));
    layout.chars.insert('é', KeyCombo::new("Digit2"));
    // ... more mappings
    
    layout
}
```

3. Add the module to `src/keyboard_layouts/mod.rs`:
```rust
pub mod fr_fr;
```

4. Register in `src/keyboard_layout.rs`:
```rust
pub static FR_FR: Lazy<KeyboardLayout> = Lazy::new(|| {
    crate::keyboard_layouts::fr_fr::create_layout()
});

// In get_layout():
"fr-FR" => Some(&FR_FR),
```

## API Reference

### `send_text_with_layout`

```rust
pub async fn send_text_with_layout(
    client: &JetKvmRpcClient,
    text: &str,
    layout_code: &str,
    delay_ms: u64,
) -> Result<()>
```

Sends text using a specific keyboard layout.

**Parameters:**
- `client`: The JetKVM RPC client
- `text`: Text to send
- `layout_code`: ISO layout code (e.g., "en-US", "es-ES")
- `delay_ms`: Delay between keystrokes in milliseconds

**Returns:** `Result<()>`

### `text_to_macro_steps`

```rust
pub fn text_to_macro_steps(
    text: &str,
    layout: &KeyboardLayout,
    delay_ms: u64,
) -> Result<Vec<MacroStep>>
```

Converts text to a series of macro steps.

**Returns:** Vector of `MacroStep` containing keys, modifiers, and delays.

## Testing

The implementation has been tested with:
- Basic ASCII text (en-US)
- Spanish accented characters (es-ES)
- Dead key sequences
- Mixed case text
- Special punctuation

## Future Enhancements

- [ ] Add more layouts (French, German, Italian, etc.)
- [ ] Support for Compose key sequences
- [ ] Layout auto-detection from system
- [ ] Custom layout definition via configuration file
- [ ] Performance optimizations for bulk text sending
