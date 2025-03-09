use anyhow::Result as AnyResult;
use jetkvm_control::jetkvm_config::JetKvmConfig;
use jetkvm_control::jetkvm_rpc_client::JetKvmRpcClient;
use jetkvm_control::keyboard::*;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> AnyResult<()> {
    // Install the default crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set debug level
        .init();

    let config = JetKvmConfig::load()?;
    let mut client = JetKvmRpcClient::new(config);

    if let Err(err) = client.connect().await {
        error!("Failed to connect to RPC server: {:?}", err);
        std::process::exit(1);
    }
    client.wait_for_channel_open().await?;
    send_windows_key(&client).await.ok();
    sleep(Duration::from_millis(250)).await;
    rpc_sendtext(&client, "notepad").await.ok();
    sleep(Duration::from_millis(250)).await;
    send_return(&client).await.ok();
    sleep(Duration::from_millis(250)).await;
    info!("sending helloworld");
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

    Ok(())
}
