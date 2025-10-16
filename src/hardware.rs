use anyhow::Result as AnyResult;
use serde_json::{json, Value};

use crate::jetkvm_rpc_client::JetKvmRpcClient;

pub async fn rpc_set_display_rotation(
    client: &JetKvmRpcClient,
    rotation: &str,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setDisplayRotation",
            json!({
                "params": {
                    "rotation": rotation
                }
            }),
        )
        .await
}

pub async fn rpc_get_display_rotation(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getDisplayRotation", json!({})).await
}

pub async fn rpc_set_backlight_settings(
    client: &JetKvmRpcClient,
    max_brightness: i32,
    dim_after: i32,
    off_after: i32,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setBacklightSettings",
            json!({
                "params": {
                    "max_brightness": max_brightness,
                    "dim_after": dim_after,
                    "off_after": off_after
                }
            }),
        )
        .await
}

pub async fn rpc_get_backlight_settings(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getBacklightSettings", json!({})).await
}
