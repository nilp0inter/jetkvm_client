use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

/// Retrieves EDID information.
pub async fn rpc_get_edid(client: &JetKvmRpcClient) -> AnyResult<String> {
    let res = client.send_rpc("getEDID", json!({})).await?;
    Ok(res
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

/// Sets the EDID data.
pub async fn rpc_set_edid(client: &JetKvmRpcClient, edid: String) -> AnyResult<Value> {
    let params = json!({ "edid": edid });
    let res = client.send_rpc("setEDID", params).await?;
    Ok(serde_json::Value::String(
        res.get("edid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    ))
}

pub async fn rpc_reboot(client: &JetKvmRpcClient, force: bool) -> AnyResult<Value> {
    let params = json!({ "force": force });
    client.send_rpc("reboot", params).await
}

pub async fn rpc_get_local_version(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getLocalVersion", json!({})).await
}

pub async fn rpc_get_update_status(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getUpdateStatus", json!({})).await
}

pub async fn rpc_try_update(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("tryUpdate", json!({})).await
}

pub async fn rpc_get_auto_update_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getAutoUpdateState", json!({})).await
}

pub async fn rpc_set_auto_update_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    let params = json!({ "enabled": enabled });
    client.send_rpc("setAutoUpdateState", params).await
}

pub async fn rpc_get_timezones(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getTimezones", json!({})).await
}
