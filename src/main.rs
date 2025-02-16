use anyhow::Result as AnyResult;
use jetkvm_control::device::{rpc_get_device_id, rpc_ping};
use jetkvm_control::jetkvm_config::JetKvmConfig;
use jetkvm_control::jetkvm_rpc_client::JetKvmRpcClient;
use jetkvm_control::mouse::rpc_abs_mouse_report;
use jetkvm_control::mouse::*;
use jetkvm_control::system::rpc_get_edid;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> AnyResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set debug level
        .init();
    info!("Starting jetkvm_control demo...");

    let config = JetKvmConfig::load()?;
    let mut client = JetKvmRpcClient::new(config);

    if let Err(err) = client.connect().await {
        error!("Failed to connect to RPC server: {:?}", err);
        std::process::exit(1);
    }

    info!("wait");
    client.wait_for_channel_open().await?;
    info!("done");
    let ping = rpc_ping(&client).await;
    info!("Ping: {:?}", ping);

    let device_id = rpc_get_device_id(&client).await;
    info!("Device ID: {:?}", device_id);
    use jetkvm_control::keyboard::*;
    send_windows_key(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    rpc_sendtext(&client, "notepad").await.ok();
    sleep(Duration::from_millis(100)).await;
    send_return(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    rpc_sendtext(&client, "Hello World").await.ok();
    sleep(Duration::from_millis(100)).await;
    send_ctrl_a(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    send_ctrl_x(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    send_ctrl_v(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    send_return(&client).await.ok();
    send_ctrl_v(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    send_return(&client).await.ok();
    send_ctrl_v(&client).await.ok();
    sleep(Duration::from_millis(100)).await;
    send_return(&client).await.ok();
    rpc_left_click(&client, 100, 500).await.ok();
    rpc_right_click(&client, 200, 500).await.ok();
    rpc_middle_click(&client, 300, 500).await.ok();
    rpc_double_click(&client, 100, 500).await.ok();

    let mouse_resp = rpc_abs_mouse_report(&client, 100, 200, 1).await;
    info!("Mouse Report: {:?}", mouse_resp);

    let edid = rpc_get_edid(&client).await;
    info!("EDID: {:?}", edid);
    let ping = rpc_ping(&client).await;
    info!("Ping: {:?}", ping);
    Ok(())
}
