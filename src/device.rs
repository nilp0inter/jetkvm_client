use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

/// Sends a "ping" request.
pub async fn rpc_ping(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("ping", json!({})).await
}

/// Retrieves the device ID.
pub async fn rpc_get_device_id(client: &JetKvmRpcClient) -> AnyResult<String> {
    let res = client.send_rpc("getDeviceID", json!({})).await?;
    Ok(res
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}
