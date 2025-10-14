use anyhow::Result as AnyResult;
use clap::Parser;
use jetkvm_client::jetkvm_rpc_client::JetKvmRpcClient;
use jetkvm_client::keyboard::*;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliConfig {
    /// The host address to connect to.
    #[arg(short = 'H', long)]
    host: String,

    /// The port number to use.
    #[arg(short = 'p', long, default_value = "80")]
    port: String,

    /// The API endpoint.
    #[arg(short = 'a', long, default_value = "/webrtc/session")]
    api: String,

    /// The password for authentication.
    #[arg(short = 'P', long)]
    password: String,

    /// The text to send.
    #[arg(short = 't', long)]
    text: String,

    /// Enable verbose logging (include logs from webrtc_sctp).
    #[arg(short = 'v', long)]
    verbose: bool,

    #[arg(short = 'C', long, default_value = "cert.pem")]
    ca_cert_path: String,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    // Install the default crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set debug level
        .init();

    let cli_config = CliConfig::parse();

    let mut client = JetKvmRpcClient::new(
        cli_config.host,
        cli_config.password,
        cli_config.api,
        false,
    );

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
    info!("sending text");
    rpc_sendtext(&client, &cli_config.text).await.ok();
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
