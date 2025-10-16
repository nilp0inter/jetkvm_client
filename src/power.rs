use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_atx_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getATXState", json!({})).await
}

pub async fn rpc_set_atx_power_action(client: &JetKvmRpcClient, action: String) -> AnyResult<Value> {
    let params = json!({ "action": action });
    client.send_rpc("setATXPowerAction", params).await
}

pub async fn rpc_get_dc_power_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getDCPowerState", json!({})).await
}

pub async fn rpc_set_dc_power_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    let params = json!({ "enabled": enabled });
    client.send_rpc("setDCPowerState", params).await
}

pub async fn rpc_set_dc_restore_state(client: &JetKvmRpcClient, state: u64) -> AnyResult<Value> {
    let params = json!({ "state": state });
    client.send_rpc("setDCRestoreState", params).await
}
