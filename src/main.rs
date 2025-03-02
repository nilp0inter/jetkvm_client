use anyhow::Result as AnyResult;
use clap::{CommandFactory, Parser};
use jetkvm_control::device::{rpc_get_device_id, rpc_ping};
use jetkvm_control::jetkvm_config::JetKvmConfig;
use jetkvm_control::jetkvm_rpc_client::JetKvmRpcClient;
use jetkvm_control::system::rpc_get_edid;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliConfig {
    /// The host address to connect to.
    #[arg(short = 'H', long)]
    host: Option<String>,

    /// The port number to use.
    #[arg(short = 'p', long)]
    port: Option<String>,

    /// The API endpoint.
    #[arg(short = 'a', long)]
    api: Option<String>,

    /// The password for authentication.
    #[arg(short = 'P', long)]
    password: Option<String>,
}

/// Loads configuration from file (or uses the default) and then applies CLI overrides.
fn load_and_override_config(cli_config: CliConfig) -> JetKvmConfig {
    let mut config = JetKvmConfig::load().unwrap_or_else(|err| {
        warn!(
            "Failed to load config.toml ({}). Using default configuration.",
            err
        );
        JetKvmConfig::default()
    });

    if let Some(host) = cli_config.host {
        config.host = host;
    }
    if let Some(port) = cli_config.port {
        config.port = port;
    }
    if let Some(api) = cli_config.api {
        config.api = api;
    }
    if let Some(password) = cli_config.password {
        config.password = password;
    }
    config
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    info!("Starting jetkvm_control demo...");

    // Parse CLI arguments.
    let cli_config = CliConfig::parse();
    info!("CLI config provided: {:?}", cli_config);

    // Load configuration from file (or default) and override with CLI options.
    let config = load_and_override_config(cli_config);

    // Validate that the critical field 'host' is set.
    if config.host.trim().is_empty() {
        eprintln!("Error: No host specified. Please set 'host' in config.toml or provide it via --host / -H.");
        CliConfig::command()
            .print_help()
            .expect("Failed to print help");
        std::process::exit(1);
    }

    // Create and use the client.
    let mut client = JetKvmRpcClient::new(config);

    if let Err(err) = client.connect().await {
        error!("Failed to connect to RPC server: {:?}", err);
        std::process::exit(1);
    }
    client.wait_for_channel_open().await?;
    let ping = rpc_ping(&client).await;
    info!("Ping: {:?}", ping);
    let device_id = rpc_get_device_id(&client).await;
    info!("Device ID: {:?}", device_id);
    let edid = rpc_get_edid(&client).await;
    info!("EDID: {:?}", edid);

    #[cfg(feature = "lua")]
    {
        use jetkvm_control::lua_engine::LuaEngine;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        let client_arc = Arc::new(Mutex::new(client));

        // Create and configure the Lua engine.
        let lua_engine = LuaEngine::new(client_arc);
        lua_engine.register_builtin_functions()?;

        // Optionally, execute a Lua script.
        let script = r#"
            print("Executing Lua script...")
            send_windows_key()
            delay(250)
            send_text("notepad")
            send_return()
            delay(250)
            send_text("Hello World!")
            send_ctrl_a()
            send_ctrl_c()
            right_click(100, 200)
        "#;

        lua_engine.exec_script(script).await?;
        info!("Lua script executed successfully.");
    }

    Ok(())
}
