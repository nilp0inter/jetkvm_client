use anyhow::Result as AnyResult;
use serde_json::{json, Value};

use crate::jetkvm_rpc_client::JetKvmRpcClient;

pub async fn rpc_get_cloud_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getCloudState", json!({})).await
}

pub async fn rpc_set_cloud_url(
    client: &JetKvmRpcClient,
    api_url: &str,
    app_url: &str,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setCloudUrl",
            json!({
                "apiUrl": api_url,
                "appUrl": app_url
            }),
        )
        .await
}

pub async fn rpc_get_tls_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getTLSState", json!({})).await
}

pub async fn rpc_set_tls_state(
    client: &JetKvmRpcClient,
    mode: &str,
    certificate: &str,
    private_key: &str,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setTLSState",
            json!({
                "state": {
                    "mode": mode,
                    "certificate": certificate,
                    "privateKey": private_key
                }
            }),
        )
        .await
}

pub async fn rpc_deregister_device(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("deregisterDevice", json!({})).await
}
