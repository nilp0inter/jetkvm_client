use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_usb_config(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getUsbConfig", json!({})).await
}

pub async fn rpc_set_usb_config(client: &JetKvmRpcClient, usb_config: Value) -> AnyResult<Value> {
    let params = json!({ "usbConfig": usb_config });
    client.send_rpc("setUsbConfig", params).await
}

pub async fn rpc_get_usb_devices(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getUsbDevices", json!({})).await
}

pub async fn rpc_set_usb_devices(client: &JetKvmRpcClient, devices: Value) -> AnyResult<Value> {
    let params = json!({ "devices": devices });
    client.send_rpc("setUsbDevices", params).await
}

pub async fn rpc_get_usb_emulation_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getUsbEmulationState", json!({})).await
}

pub async fn rpc_set_usb_emulation_state(client: &JetKvmRpcClient, enabled: bool) -> AnyResult<Value> {
    let params = json!({ "enabled": enabled });
    client.send_rpc("setUsbEmulationState", params).await
}
