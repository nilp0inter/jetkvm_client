use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_jiggler_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getJigglerState", json!({})).await
}

pub async fn rpc_set_jiggler_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    let params = json!({ "enabled": enabled });
    client.send_rpc("setJigglerState", params).await
}

pub async fn rpc_get_jiggler_config(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getJigglerConfig", json!({})).await
}

pub async fn rpc_set_jiggler_config(client: &JetKvmRpcClient, jiggler_config: Value) -> AnyResult<Value> {
    let params = json!({ "jigglerConfig": jiggler_config });
    client.send_rpc("setJigglerConfig", params).await
}
