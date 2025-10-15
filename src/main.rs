use anyhow::Result as AnyResult;
use base64::{engine::general_purpose, Engine as _};
use clap::{CommandFactory, Parser};
use jetkvm_client::device::{rpc_get_device_id, rpc_ping};
use jetkvm_client::jetkvm_rpc_client::{JetKvmRpcClient, SignalingMethod};
use jetkvm_client::keyboard::{
    rpc_keyboard_report, rpc_sendtext, send_ctrl_a, send_ctrl_c, send_ctrl_v, send_ctrl_x,
    send_return, send_text_with_layout, send_windows_key,
};
use jetkvm_client::mouse::{
    rpc_abs_mouse_report, rpc_double_click, rpc_left_click, rpc_middle_click, rpc_move_mouse,
    rpc_right_click, rpc_wheel_report,
};
use jetkvm_client::system::{rpc_get_edid, rpc_set_edid};
use serde_json::json;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

    /// Enable debug logging.
    #[arg(short = 'd', long)]
    debug: bool,

    #[arg(short = 'C', long, default_value = "cert.pem")]
    ca_cert_path: String,

    /// The signaling method to use.
    #[arg(long, value_enum, default_value_t = SignalingMethod::Auto)]
    signaling_method: SignalingMethod,

    /// The sequence of commands to execute.
    #[arg(required = true, num_args = 1.., trailing_var_arg = true)]
    commands: Vec<String>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Sends a "ping" request.
    Ping,
    /// Retrieves the device ID.
    #[command(name = "get-device-id")]
    GetDeviceId,
    /// Retrieves EDID information.
    #[command(name = "get-edid")]
    GetEdid,
    /// Sets the EDID data.
    #[command(name = "set-edid")]
    SetEdid { edid: String },
    /// Sends a keyboard report with the given modifier and keys.
    #[command(name = "keyboard-report")]
    KeyboardReport {
        #[arg(long)]
        modifier: u64,
        #[arg(long, num_args = 0..)]
        keys: Vec<u8>,
    },
    /// Sends text as a series of keyboard events (US ASCII only).
    #[command(name = "sendtext")]
    Sendtext { text: String },
    /// Sends text using a specific keyboard layout (supports accents and special characters).
    #[command(name = "send-text-with-layout")]
    SendTextWithLayout {
        text: String,
        #[arg(long, default_value = "en-US")]
        layout: String,
        #[arg(long, default_value = "20")]
        delay: u64,
    },
    /// Sends a Return (Enter) key press.
    #[command(name = "send-return")]
    SendReturn,
    /// Sends a Ctrl-C keyboard event.
    #[command(name = "send-ctrl-c")]
    SendCtrlC,
    /// Sends a Ctrl-V keyboard event.
    #[command(name = "send-ctrl-v")]
    SendCtrlV,
    /// Sends a Ctrl-X keyboard event.
    #[command(name = "send-ctrl-x")]
    SendCtrlX,
    /// Sends a Ctrl-A keyboard event.
    #[command(name = "send-ctrl-a")]
    SendCtrlA,
    /// Sends a Windows key press.
    #[command(name = "send-windows-key")]
    SendWindowsKey,
    /// Sends an absolute mouse report with x, y coordinates and button state.
    #[command(name = "abs-mouse-report")]
    AbsMouseReport { x: i64, y: i64, buttons: u64 },
    /// Sends a wheel report with the given wheelY value.
    #[command(name = "wheel-report")]
    WheelReport { wheel_y: i64 },
    /// Moves the mouse to the specified absolute coordinates.
    #[command(name = "move-mouse")]
    MoveMouse { x: i64, y: i64 },
    /// Simulates a left mouse click at the specified coordinates.
    #[command(name = "left-click")]
    LeftClick { x: i64, y: i64 },
    /// Simulates a right mouse click at the specified coordinates.
    #[command(name = "right-click")]
    RightClick { x: i64, y: i64 },
    /// Simulates a middle mouse click at the specified coordinates.
    #[command(name = "middle-click")]
    MiddleClick { x: i64, y: i64 },
    /// Simulates a double left click at the specified coordinates.
    #[command(name = "double-click")]
    DoubleClick { x: i64, y: i64 },
    /// Captures a screenshot as PNG (returns base64 encoded data URL).
    #[command(name = "screenshot")]
    Screenshot,
    /// Waits for the specified number of milliseconds.
    #[command(name = "wait")]
    Wait { milliseconds: u64 },
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    // Install the default crypto provider for rustls
    #[cfg(feature = "tls")]
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    // Parse CLI arguments.
    let cli = Cli::parse();

    if cli.debug {
        registry()
            .with(EnvFilter::new("debug"))
            .with(fmt::layer().with_writer(std::io::stderr))
            .init();
        info!("Starting jetkvm_client...");
    }

    // Create and connect the client.
    let mut client =
        JetKvmRpcClient::new(cli.host, cli.password, cli.api, false, cli.signaling_method);
    if let Err(err) = client.connect().await {
        let error_json = json!({ "error": format!("Failed to connect to RPC server: {:?}", err) });
        println!("{}", serde_json::to_string(&error_json)?);
    } else {
        client.wait_for_channel_open().await?;
    }

    let mut command_args = cli.commands.into_iter();
    while let Some(arg) = command_args.next() {
        let mut sub_args = vec![arg];
        while let Some(next_arg) = command_args.next() {
            if Commands::command()
                .get_subcommands()
                .any(|c| c.get_name() == next_arg)
            {
                // This is a new command, so we need to parse the previous one
                command_args = vec![next_arg]
                    .into_iter()
                    .chain(command_args)
                    .collect::<Vec<_>>()
                    .into_iter();
                break;
            }
            sub_args.push(next_arg);
        }

        let command = match Commands::try_parse_from(
            std::iter::once("jetkvm_client".to_string()).chain(sub_args.clone().into_iter()),
        ) {
            Ok(command) => command,
            Err(e) => {
                e.exit();
            }
        };

        let command_info = json!({
            "command": sub_args[0],
            "params": if sub_args.len() > 1 { json!(sub_args[1..].to_vec()) } else { json!([]) }
        });

        let result = match command {
            Commands::Ping => rpc_ping(&client).await,
            Commands::GetDeviceId => rpc_get_device_id(&client).await.map(|r| json!(r)),
            Commands::GetEdid => rpc_get_edid(&client).await.map(|r| json!(r)),
            Commands::SetEdid { edid } => rpc_set_edid(&client, edid)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::KeyboardReport { modifier, keys } => {
                rpc_keyboard_report(&client, modifier, keys)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::Sendtext { text } => rpc_sendtext(&client, &text)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendTextWithLayout {
                text,
                layout,
                delay,
            } => send_text_with_layout(&client, &text, &layout, delay)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendReturn => send_return(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendCtrlC => send_ctrl_c(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendCtrlV => send_ctrl_v(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendCtrlX => send_ctrl_x(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendCtrlA => send_ctrl_a(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SendWindowsKey => send_windows_key(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::AbsMouseReport { x, y, buttons } => {
                rpc_abs_mouse_report(&client, x, y, buttons)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::WheelReport { wheel_y } => rpc_wheel_report(&client, wheel_y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::MoveMouse { x, y } => rpc_move_mouse(&client, x, y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::LeftClick { x, y } => rpc_left_click(&client, x, y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::RightClick { x, y } => rpc_right_click(&client, x, y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::MiddleClick { x, y } => rpc_middle_click(&client, x, y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::DoubleClick { x, y } => rpc_double_click(&client, x, y)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::Screenshot => {
                client
                    .video_capture
                    .capture_screenshot_png()
                    .await
                    .map(|png_data| {
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        let data_url = format!("data:image/png;base64,{}", base64_data);
                        json!({
                            "status": "ok",
                            "format": "png",
                            "size": png_data.len(),
                            "data": data_url
                        })
                    })
            }
            Commands::Wait { milliseconds } => {
                tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
                Ok(json!({ "status": "ok" }))
            }
        };

        match result {
            Ok(value) => {
                let result_json = json!({
                    "command": command_info["command"],
                    "params": command_info["params"],
                    "result": value
                });
                println!("{}", serde_json::to_string(&result_json)?);
            }
            Err(e) => {
                let error_json = json!({
                    "command": command_info["command"],
                    "params": command_info["params"],
                    "error": format!("{}", e)
                });
                println!("{}", serde_json::to_string(&error_json)?);
            }
        }
    }

    Ok(())
}
