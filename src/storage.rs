use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};

pub async fn rpc_get_virtual_media_state(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getVirtualMediaState", json!({})).await
}

pub async fn rpc_mount_with_http(client: &JetKvmRpcClient, url: String, mode: String) -> AnyResult<Value> {
    let params = json!({
        "url": url,
        "mode": mode,
    });
    client.send_rpc("mountWithHTTP", params).await
}

pub async fn rpc_mount_with_storage(client: &JetKvmRpcClient, filename: String, mode: String) -> AnyResult<Value> {
    let params = json!({
        "filename": filename,
        "mode": mode,
    });
    client.send_rpc("mountWithStorage", params).await
}

pub async fn rpc_unmount_image(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("unmountImage", json!({})).await
}

pub async fn rpc_list_storage_files(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("listStorageFiles", json!({})).await
}

pub async fn rpc_get_storage_space(client: &JetKvmRpcClient) -> AnyResult<Value> {
    client.send_rpc("getStorageSpace", json!({})).await
}

pub async fn rpc_delete_storage_file(client: &JetKvmRpcClient, filename: String) -> AnyResult<Value> {
    let params = json!({ "filename": filename });
    client.send_rpc("deleteStorageFile", params).await
}

pub async fn rpc_start_storage_file_upload(client: &JetKvmRpcClient, filename: String, size: u64) -> AnyResult<Value> {
    let params = json!({
        "filename": filename,
        "size": size,
    });
    client.send_rpc("startStorageFileUpload", params).await
}
