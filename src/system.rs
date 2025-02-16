use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

/// Retrieves EDID information.
pub async fn rpc_get_edid(client: &JetKvmRpcClient) -> AnyResult<String> {
    let res = client.send_rpc("getEDID", json!({})).await?;
    Ok(res
        .get("edid")
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
