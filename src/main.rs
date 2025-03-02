use anyhow::Result as AnyResult;
use jetkvm_control::device::{rpc_get_device_id, rpc_ping};
use jetkvm_control::jetkvm_config::JetKvmConfig;
use jetkvm_control::jetkvm_rpc_client::JetKvmRpcClient;
use jetkvm_control::system::rpc_get_edid;

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
    client.wait_for_channel_open().await?;
    let ping = rpc_ping(&client).await;
    info!("Ping: {:?}", ping);
    let device_id = rpc_get_device_id(&client).await;
    info!("Device ID: {:?}", device_id);

    let edid = rpc_get_edid(&client).await;
    info!("EDID: {:?}", edid);

    #[cfg(feature = "lua")]
    {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use jetkvm_control::lua_engine::LuaEngine;
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
