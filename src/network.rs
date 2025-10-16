use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_network_settings(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getNetworkSettings", json!({})).await
}

pub async fn rpc_set_network_settings(client: &JetKvmRpcClient, settings: Value) -> AnyResult<Value> {
    let params = json!({ "settings": settings });
    client.send_rpc("setNetworkSettings", params).await
}

pub async fn rpc_get_network_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getNetworkState", json!({})).await
}

pub async fn rpc_renew_dhcp_lease(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("renewDHCPLease", json!({})).await
}
