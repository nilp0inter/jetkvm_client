use anyhow::Result as AnyResult;
use serde_json::{json, Value};

use crate::jetkvm_rpc_client::JetKvmRpcClient;

pub async fn rpc_get_dev_mode_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getDevModeState", json!({})).await
}

pub async fn rpc_set_dev_mode_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    client
        .send_rpc(
            "setDevModeState",
            json!({
                "enabled": enabled
            }),
        )
        .await
}

pub async fn rpc_get_ssh_key_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getSSHKeyState", json!({})).await
}

pub async fn rpc_set_ssh_key_state(client: &JetKvmRpcClient, ssh_key: &str) -> AnyResult<Value> {
    client
        .send_rpc(
            "setSSHKeyState",
            json!({
                "sshKey": ssh_key
            }),
        )
        .await
}

pub async fn rpc_get_dev_channel_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getDevChannelState", json!({})).await
}

pub async fn rpc_set_dev_channel_state(
    client: &JetKvmRpcClient,
    enabled: bool,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setDevChannelState",
            json!({
                "enabled": enabled
            }),
        )
        .await
}

pub async fn rpc_get_local_loopback_only(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getLocalLoopbackOnly", json!({})).await
}

pub async fn rpc_set_local_loopback_only(
    client: &JetKvmRpcClient,
    enabled: bool,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setLocalLoopbackOnly",
            json!({
                "enabled": enabled
            }),
        )
        .await
}

pub async fn rpc_reset_config(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("resetConfig", json!({})).await
}
