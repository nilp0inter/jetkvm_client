use anyhow::Result as AnyResult;
use serde_json::{json, Value};

use crate::jetkvm_rpc_client::JetKvmRpcClient;

pub async fn rpc_get_active_extension(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getActiveExtension", json!({})).await
}

pub async fn rpc_set_active_extension(
    client: &JetKvmRpcClient,
    extension_id: &str,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setActiveExtension",
            json!({
                "extensionId": extension_id
            }),
        )
        .await
}

pub async fn rpc_get_serial_settings(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getSerialSettings", json!({})).await
}

pub async fn rpc_set_serial_settings(
    client: &JetKvmRpcClient,
    baud_rate: &str,
    data_bits: &str,
    stop_bits: &str,
    parity: &str,
) -> AnyResult<Value> {
    client
        .send_rpc(
            "setSerialSettings",
            json!({
                "settings": {
                    "baudRate": baud_rate,
                    "dataBits": data_bits,
                    "stopBits": stop_bits,
                    "parity": parity
                }
            }),
        )
        .await
}

pub async fn rpc_set_atx_power_action(client: &JetKvmRpcClient, action: &str) -> AnyResult<Value> {
    client
        .send_rpc(
            "setATXPowerAction",
            json!({
                "action": action
            }),
        )
        .await
}

pub async fn rpc_get_dc_power_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getDCPowerState", json!({})).await
}

pub async fn rpc_set_dc_power_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    client
        .send_rpc(
            "setDCPowerState",
            json!({
                "enabled": enabled
            }),
        )
        .await
}

pub async fn rpc_set_dc_restore_state(client: &JetKvmRpcClient, state: u8) -> AnyResult<Value> {
    client
        .send_rpc(
            "setDCRestoreState",
            json!({
                "state": state
            }),
        )
        .await
}
            
            
