use anyhow::Result as AnyResult;
use clap::Parser;
use jetkvm_client::jetkvm_rpc_client::JetKvmRpcClient;
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

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

    /// Enable verbose logging (include logs from webrtc_sctp).
    #[arg(short = 'v', long)]
    verbose: bool,

    #[arg(short = 'C', long, default_value = "cert.pem")]
    ca_cert_path: String,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    // Install the default crypto provider for rustls
    #[cfg(feature = "tls")]
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    // Parse CLI arguments.
    let cli_config = CliConfig::parse();
    info!("CLI config provided: {:?}", cli_config);

    // Build a filter string: by default, disable webrtc_sctp logging,
    // but if verbose is enabled, include all logs.
    let filter_directive = if cli_config.verbose {
        "debug"
    } else {
        "debug,\
         webrtc_sctp=off,\
         webrtc::peer_connection=off,\
         webrtc_dtls=off,\
         webrtc_mdns=off,\
         hyper_util::client=off,\
         webrtc_data::data_channel=off,\
         webrtc_ice=off"
    };

    // Initialize tracing subscriber with the constructed filter.
    // Create an EnvFilter using the directive.
    let env_filter = EnvFilter::new(filter_directive);

    // Build a subscriber with the filter layer and formatting layer.
    registry().with(env_filter).with(fmt::layer()).init();
    info!("Starting jetkvm_client demo...");

    // Create and connect the client.
    let mut client =
        JetKvmRpcClient::new(cli_config.host, cli_config.password, cli_config.api, false);
    if let Err(err) = client.connect().await {
        error!("Failed to connect to RPC server: {:?}", err);
        std::process::exit(1);
    }
    client.wait_for_channel_open().await?;

    // Normal mode: if the "lua" feature is not enabled, perform normal actions.
    {
        use jetkvm_client::device::{rpc_get_device_id, rpc_ping};
        use jetkvm_client::system::rpc_get_edid;

        let ping = rpc_ping(&client).await;
        info!("Ping: {:?}", ping);
        let device_id = rpc_get_device_id(&client).await;
        info!("Device ID: {:?}", device_id);
        let edid = rpc_get_edid(&client).await;
        info!("EDID: {:?}", edid);
        // Logout after Lua execution
        client.logout().await?;
    }

    Ok(())
}
