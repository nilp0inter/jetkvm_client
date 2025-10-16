use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_wake_on_lan_devices(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getWakeOnLanDevices", json!({})).await
}

pub async fn rpc_set_wake_on_lan_devices(client: &JetKvmRpcClient, params: Value) -> AnyResult<Value> {
    client.send_rpc("setWakeOnLanDevices", params).await
}

pub async fn rpc_send_wol_magic_packet(client: &JetKvmRpcClient, mac_address: String) -> AnyResult<Value> {
    let params = json!({ "macAddress": mac_address });
    client.send_rpc("sendWOLMagicPacket", params).await
}
