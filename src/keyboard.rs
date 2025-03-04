use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::debug;

/// Sends a keyboard report with the given modifier and keys.
pub async fn rpc_keyboard_report(
    client: &JetKvmRpcClient,
    modifier: u64,
    keys: Vec<u8>,
) -> AnyResult<Value> {
    let params = json!({
        "modifier": modifier,
        "keys": keys,
    });
    client.send_rpc("keyboardReport", params).await
}

/// Convert an ASCII character into a (modifier, keycode) pair, following HID usage tables.
///
/// For letters:
/// - Lowercase: modifier = 0, keycode = 0x04 + (c - 'a').
/// - Uppercase: modifier = 0x02 (shift), keycode = same as lowercase.
///
/// For digits:
/// - '1' to '9': keycodes 0x1E to 0x26 respectively,
/// - '0': keycode 0x27.
///
/// For space:
/// - Keycode is 0x2C with no modifier.
///
/// For common punctuation and symbols, the mapping is defined in a static table.
///
/// Returns `None` if the character is not supported.
fn char_to_hid(c: char) -> Option<(u8, u8)> {
    if c.is_ascii_alphabetic() {
        let shift = if c.is_ascii_uppercase() { 0x02 } else { 0 };
        Some((shift, (c.to_ascii_lowercase() as u8) - b'a' + 0x04))
    } else if c.is_ascii_digit() {
        if c == '0' {
            Some((0, 0x27))
        } else {
            Some((0, (c as u8) - b'1' + 0x1E))
        }
    } else if c == ' ' {
        // HID usage for space.
        Some((0, 0x2C))
    } else {
        // Mapping for additional punctuation and symbols.
        const MAP: &[(char, (u8, u8))] = &[
            ('!', (0x02, 0x1E)), // Shift + '1'
            ('@', (0x02, 0x1F)), // Shift + '2'
            ('#', (0x02, 0x20)), // Shift + '3'
            ('$', (0x02, 0x21)), // Shift + '4'
            ('%', (0x02, 0x22)), // Shift + '5'
            ('^', (0x02, 0x23)), // Shift + '6'
            ('&', (0x02, 0x24)), // Shift + '7'
            ('*', (0x02, 0x25)), // Shift + '8'
            ('(', (0x02, 0x26)), // Shift + '9'
            (')', (0x02, 0x27)), // Shift + '0'
            ('-', (0, 0x2D)),
            ('_', (0x02, 0x2D)),
            ('=', (0, 0x2E)),
            ('+', (0x02, 0x2E)),
            ('[', (0, 0x2F)),
            ('{', (0x02, 0x2F)),
            (']', (0, 0x30)),
            ('}', (0x02, 0x30)),
            ('\\', (0, 0x31)),
            ('|', (0x02, 0x31)),
            (';', (0, 0x33)),
            (':', (0x02, 0x33)),
            ('\'', (0, 0x34)),
            ('"', (0x02, 0x34)),
            ('`', (0, 0x35)),
            ('~', (0x02, 0x35)),
            (',', (0, 0x36)),
            ('<', (0x02, 0x36)),
            ('.', (0, 0x37)),
            ('>', (0x02, 0x37)),
            ('/', (0, 0x38)),
            ('?', (0x02, 0x38)),
        ];
        MAP.iter()
            .find_map(|&(ch, pair)| if ch == c { Some(pair) } else { None })
    }
}

/// Sends text as a series of keyboard events (press and release) over the JSON‑RPC channel.
/// It iterates over each character in the provided text.
pub async fn rpc_sendtext(
    client: &crate::jetkvm_rpc_client::JetKvmRpcClient,
    text: &str,
) -> AnyResult<()> {
    // For each character, simulate a key press then a key release.
    for c in text.chars() {
        if let Some((modifier, keycode)) = char_to_hid(c) {
            // Press key:
            crate::keyboard::rpc_keyboard_report(client, modifier as u64, vec![keycode]).await?;
            // Wait a short period (e.g., 50 ms)
            sleep(Duration::from_millis(10)).await;
            // Release key:
            crate::keyboard::rpc_keyboard_report(client, 0, vec![]).await?;
            sleep(Duration::from_millis(10)).await;
        } else {
            debug!("Unsupported character: {}", c);
        }
    }
    Ok(())
}

/// Sends a Return (Enter) key press then releases it.
pub async fn send_return(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Return (keycode 0x28)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [0x28],
                "modifier": 0
            }),
        )
        .await?;

    // Wait a short period to simulate a key press duration.
    sleep(Duration::from_millis(100)).await;

    // Release all keys.
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0
            }),
        )
        .await?;

    Ok(())
}

/// Sends a Ctrl-C keyboard event: press Ctrl-C, wait, then release.
pub async fn send_ctrl_c(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Ctrl-C: 'C' has HID code 0x06, with Ctrl modifier (0x01)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [0x06],
                "modifier": 0x01,
            }),
        )
        .await?;
    sleep(Duration::from_millis(100)).await;
    // Release keys.
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0,
            }),
        )
        .await?;
    Ok(())
}

/// Sends a Ctrl-V keyboard event: press Ctrl-V, wait, then release.
pub async fn send_ctrl_v(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Ctrl-V: 'V' has HID code 0x19, with Ctrl modifier (0x01)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [0x19],
                "modifier": 0x01,
            }),
        )
        .await?;
    sleep(Duration::from_millis(100)).await;
    // Release keys.
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0,
            }),
        )
        .await?;
    Ok(())
}

/// Sends a Ctrl-X keyboard event: press Ctrl-X, wait, then release.
pub async fn send_ctrl_x(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Ctrl-X: 'X' has HID code 0x1B, with Ctrl modifier (0x01)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [0x1B],
                "modifier": 0x01,
            }),
        )
        .await?;
    sleep(Duration::from_millis(100)).await;
    // Release keys.
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0,
            }),
        )
        .await?;
    Ok(())
}

/// Sends a Ctrl-A keyboard event: press Ctrl-A, wait, then release.
pub async fn send_ctrl_a(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Ctrl-A: modifier 0x01 (Ctrl) and keycode 0x04 ('A')
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [0x04],
                "modifier": 0x01
            }),
        )
        .await?;

    // Wait 100 milliseconds.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Release keys.
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0
            }),
        )
        .await?;

    Ok(())
}

/// Sends a Windows key press (using the left GUI key) then releases it.
pub async fn send_windows_key(client: &crate::jetkvm_rpc_client::JetKvmRpcClient) -> AnyResult<()> {
    // Press Windows key (left GUI; modifier 0x08)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0x08
            }),
        )
        .await?;

    // Wait a short period (e.g., 100 ms)
    sleep(Duration::from_millis(100)).await;

    // Release keys (modifier 0)
    client
        .send_rpc(
            "keyboardReport",
            json!({
                "keys": [],
                "modifier": 0
            }),
        )
        .await?;

    Ok(())
}

/// Represents a key combination for a remote KVM device.
/// 
/// This struct defines the information required to simulate a key press event:
/// - `modifier`: A bitmask representing modifier keys (e.g., Ctrl, Alt).
/// - `keys`: A list of key codes to be pressed.
/// - `hold`: The duration (in milliseconds) to hold the key press (defaults to 100 ms).
/// - `wait`: The delay (in milliseconds) after releasing the keys before proceeding to the next combination (defaults to 10 ms).
#[derive(Debug)]
struct KeyCombo {
    modifier: u64,
    keys: Vec<u8>,
    hold: u64, // how long to hold the key press (in ms), defaults to 100 ms
    wait: u64, // delay after releasing keys before next combo (in ms), defaults to 10 ms
}

/// Sends a series of key combinations to the remote KVM device via RPC.
///
/// This asynchronous function processes each key combination in the provided list by:
/// 1. Sending a `keyboardReport` RPC that presses the specified keys along with any modifier keys.
/// 2. Waiting for the duration specified in `combo.hold` to simulate holding the keys down.
/// 3. Sending another `keyboardReport` RPC to release all keys.
/// 4. Waiting for the duration specified in `combo.wait` before moving to the next combination.
///
/// # Parameters
/// - `client`: A reference to the `JetKvmRpcClient` that is used to send RPC commands.
/// - `combos`: A vector of `KeyCombo` structs, where each struct contains:
///   - `modifier`: The modifier key(s) (like Ctrl, Alt, etc.) to apply.
///   - `keys`: The list of keys to be pressed.
///   - `hold`: The duration (in milliseconds) to hold the keys down.
///   - `wait`: The duration (in milliseconds) to wait after releasing the keys.
///
/// # Returns
/// Returns `Ok(())` if all key combinations are successfully processed, or an error if any RPC call fails.
async fn send_key_combinations(client: &JetKvmRpcClient, combos: Vec<KeyCombo>) -> AnyResult<()> {
    for combo in combos {
        // Press the keys with the specified modifier.
        client
            .send_rpc(
                "keyboardReport",
                json!({
                    "modifier": combo.modifier,
                    "keys": combo.keys,
                }),
            )
            .await?;
        // Hold the key for the specified duration.
        sleep(Duration::from_millis(combo.hold)).await;
        // Release all keys.
        client
            .send_rpc(
                "keyboardReport",
                json!({
                    "modifier": 0,
                    "keys": [],
                }),
            )
            .await?;
        // Wait after releasing keys before proceeding.
        sleep(Duration::from_millis(combo.wait)).await;
    }
    Ok(())
}

/// Registers keyboard functions to the provided Lua context.
#[cfg(feature = "lua")]
use mlua::prelude::*;
#[cfg(feature = "lua")]
pub fn register_lua(lua: &Lua, client: Arc<Mutex<JetKvmRpcClient>>) -> LuaResult<()> {
    let send_return_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_return(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_return", send_return_fn)?;

    let send_ctrl_a_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_ctrl_a(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_ctrl_a", send_ctrl_a_fn)?;

    let send_ctrl_v_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_ctrl_v(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_ctrl_v", send_ctrl_v_fn)?;

    let send_ctrl_x_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_ctrl_x(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_ctrl_x", send_ctrl_x_fn)?;

    let send_ctrl_c_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_ctrl_c(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_ctrl_c", send_ctrl_c_fn)?;

    let send_windows_key_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, ()| {
            let client = client.clone();
            async move {
                send_windows_key(&*client.lock().await)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_windows_key", send_windows_key_fn)?;

    let send_text_fn = {
        let client = client.clone();
        lua.create_async_function(move |_, text: String| {
            let client = client.clone();
            async move {
                rpc_sendtext(&*client.lock().await, &text)
                    .await
                    .map_err(mlua::Error::external)
            }
        })?
    };
    lua.globals().set("send_text", send_text_fn)?;

    let send_key_combinations_fn = {
        let value = client.clone();
        // The async function returns a Result; we unwrap it here.
        lua.create_async_function(move |_, combos: mlua::Table| {
            let client = value.clone();
            async move {
                let mut key_combos = Vec::new();
                // Expecting combos to be an array of tables.
                for pair in combos.sequence_values::<mlua::Table>() {
                    let combo_table = pair?;
                    let modifier: u64 = combo_table.get("modifier")?;
                    let keys: Vec<u8> = combo_table.get("keys")?;
                    // Optional fields: default to hold = 100 ms and wait = 10 ms.
                    let hold: u64 = combo_table.get("hold").unwrap_or(100);
                    let wait: u64 = combo_table.get("wait").unwrap_or(10);
                    key_combos.push(KeyCombo {
                        modifier,
                        keys,
                        hold,
                        wait,
                    });
                }
                send_key_combinations(&*client.lock().await, key_combos)
                    .await
                    .map_err(mlua::Error::external)
            }
        })
    }?;

    lua.globals()
        .set("send_key_combinations", send_key_combinations_fn)?;

    Ok(())
}
