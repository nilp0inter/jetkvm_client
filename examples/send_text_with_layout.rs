use anyhow::Result;
use clap::Parser;
use jetkvm_client::jetkvm_rpc_client::{JetKvmRpcClient, SignalingMethod};
use jetkvm_client::keyboard::send_text_with_layout;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliConfig {
    #[arg(short = 'H', long)]
    host: String,

    #[arg(short = 'a', long, default_value = "/webrtc/session")]
    api: String,

    #[arg(short = 'P', long)]
    password: String,

    #[arg(short = 'l', long, default_value = "en-US")]
    layout: String,

    #[arg(long, value_enum, default_value_t = SignalingMethod::Auto)]
    signaling_method: SignalingMethod,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli_config = CliConfig::parse();

    let mut client = JetKvmRpcClient::new(
        cli_config.host.clone(),
        cli_config.password.clone(),
        cli_config.api.clone(),
        false,
        cli_config.signaling_method.clone(),
    );

    tracing::info!("Connecting to JetKVM device...");
    client.connect().await?;
    client.wait_for_channel_open().await?;

    tracing::info!(
        "Connected! Sending text with {} layout...",
        cli_config.layout
    );
    send_text_with_layout(&client, "Hello, World!\n", &cli_config.layout, 20).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    if cli_config.layout == "es-ES" {
        tracing::info!("Sending Spanish text with es-ES layout...");
        send_text_with_layout(&client, "¡Hola! ¿Cómo estás? ñ á é í ó ú\n", "es-ES", 20).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        tracing::info!("Sending accented characters...");
        send_text_with_layout(&client, "Ñoño comió açaí.\n", "es-ES", 20).await?;
    }

    tracing::info!("Done!");

    Ok(())
}
