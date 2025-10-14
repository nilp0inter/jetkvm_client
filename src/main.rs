use anyhow::Result as AnyResult;
use clap::{CommandFactory, Parser};
use jetkvm_client::jetkvm_config::JetKvmConfig;
use jetkvm_client::jetkvm_rpc_client::JetKvmRpcClient;
use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

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

    /// Enable verbose logging (include logs from webrtc_sctp).
    #[arg(short = 'v', long)]
    verbose: bool,

    // When the "lua" feature is enabled, the first positional argument is the Lua script path.
    #[cfg(feature = "lua")]
    /// Path to the Lua script to execute.
    #[arg(required = false, index = 1, default_value = "", num_args = 0..=1)]
    lua_script: String,

    #[arg(short = 'C', long, default_value = "cert.pem")]
    ca_cert_path: Option<String>,

    /// Initialize or edit the jetkvm_client.toml interactively.
    #[arg(short = 'c', long = "config_init")]
    config_init: bool,
    
}

/// Loads configuration from file (or uses the default) and then applies CLI overrides.
fn load_and_override_config(cli_config: &CliConfig) -> JetKvmConfig {
    let (mut config, _, _) = JetKvmConfig::load().unwrap_or_else(|err| {
        warn!(
            "Failed to load jetkvm_client.toml ({}). Using default configuration.",
            err
        );
        (JetKvmConfig::default(),"".to_string(),true)
    });

    if let Some(host) = &cli_config.host {
        config.host = host.clone();
    }
    if let Some(port) = &cli_config.port {
        config.port = port.clone();
    }
    if let Some(api) = &cli_config.api {
        config.api = api.clone();
    }
    if let Some(password) = &cli_config.password {
        config.password = password.clone();
    }
    if let Some(ca_cert_path) = &cli_config.ca_cert_path {
        config.ca_cert_path = ca_cert_path.clone();
    }
    config
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
    if cli_config.config_init {
        jetkvm_client::jetkvm_config::interactive_config_location().await?;
        return Ok(());
    }
    #[cfg(feature = "lua")]
    {
        if cli_config.lua_script.is_empty() {
            eprintln!("Error: You must provide a Lua script when using the lua feature.");
            // Print help and exit.
            CliConfig::command().print_help().expect("failed to print help");
            println!(); // newline after help
            std::process::exit(1);
        }
    }

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

    // Load configuration from file (or default) and override with CLI options.
    let config = load_and_override_config(&cli_config);

    // Validate that the critical field 'host' is set.
    if config.host.trim().is_empty() {
        eprintln!("Error: No host specified. Please set 'host' in jetkvm_client.toml or provide it via --host / -H.");
        CliConfig::command()
            .print_help()
            .expect("Failed to print help");
        std::process::exit(1);
    }

    // Create and connect the client.
    let mut client = JetKvmRpcClient::new(config.clone());
    if let Err(err) = client.connect().await {
        error!("Failed to connect to RPC server: {:?}", err);
        std::process::exit(1);
    }
    client.wait_for_channel_open().await?;

    // Lua mode: if the "lua" feature is enabled, read and execute the provided Lua script.
    #[cfg(feature = "lua")]
    {
        use jetkvm_client::lua_engine::LuaEngine;
        use std::sync::Arc;
        use tokio::sync::Mutex;


// Resolve lua_script to a full absolute path
let lua_script_path = tokio::fs::canonicalize(&cli_config.lua_script).await.map_err(|e| {
    anyhow::anyhow!(
        "Error resolving Lua script path '{}'.\nArguments: {:?}\nCurrent directory: '{}'\nError: {}",
        &cli_config.lua_script,
        std::env::args().collect::<Vec<_>>(),
        std::env::current_dir().unwrap().display(),
        e
    )
})?;

let script = tokio::fs::read_to_string(&lua_script_path).await.map_err(|e| {
    anyhow::anyhow!(
        "Error reading Lua script from '{}'. Arguments passed: {:?}. Error details: {}",
        cli_config.lua_script,
        std::env::args().collect::<Vec<_>>(),
        e
    )
})?;

println!("Current working directory: {}", std::env::current_dir()?.display());
info!("Executing Lua script from {}", &cli_config.lua_script);

        // Wrap the client in an Arc/Mutex for the Lua engine.
        let client_arc = Arc::new(Mutex::new(client));
        let lua_engine = LuaEngine::new(client_arc.clone());
        lua_engine.register_builtin_functions()?;

        let config_clone = config.clone(); // ✅ Clone before moving

        lua_engine.lua().globals().set("HOST", config_clone.host)?;
        lua_engine.lua().globals().set("PORT", config_clone.port)?;
        lua_engine
            .lua()
            .globals()
            .set("PASSWORD", config_clone.password)?;
        lua_engine
            .lua()
            .globals()
            .set("CERT_PATH", config_clone.ca_cert_path)?;

        lua_engine.exec_script(&script).await?;
        info!("Lua script executed successfully.");
        // Logout after Lua execution
        client_arc.lock().await.logout().await?;
    }

    // Normal mode: if the "lua" feature is not enabled, perform normal actions.
    #[cfg(not(feature = "lua"))]
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
