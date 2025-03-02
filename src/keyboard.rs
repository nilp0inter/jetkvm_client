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
/// Currently supports letters and digits only.
/// - For lowercase letters, modifier = 0, keycode = 0x04 + (c - 'a').
/// - For uppercase letters, modifier = 0x02 (shift), keycode = same as lowercase.
/// - For digits: '1' to '9' map to 0x1E..0x26, '0' maps to 0x27.
fn char_to_hid(c: char) -> Option<(u8, u8)> {
    if c.is_ascii_alphabetic() {
        let base = 0x04;
        if c.is_ascii_lowercase() {
            Some((0, (c as u8) - b'a' + base))
        } else {
            // Uppercase letters require SHIFT (modifier 0x02)
            Some((0x02, (c.to_ascii_lowercase() as u8) - b'a' + base))
        }
    } else if c.is_ascii_digit() {
        if c == '0' {
            Some((0, 0x27))
        } else {
            Some((0, (c as u8) - b'1' + 0x1E))
        }
    } else if c == ' ' {
        // HID usage for space is 0x2C, no modifier.
        Some((0, 0x2C))
    } else {
        // Extend with additional mappings as needed.
        None
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

    Ok(())
}
