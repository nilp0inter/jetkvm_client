use anyhow::Result as AnyResult;
use base64::{engine::general_purpose, Engine as _};
use clap::{ArgAction, CommandFactory, Parser};
use std::fs::File;
use std::io::Write;
use jetkvm_client::advanced::{
    rpc_get_dev_channel_state, rpc_get_dev_mode_state, rpc_get_local_loopback_only,
    rpc_get_ssh_key_state, rpc_reset_config, rpc_set_dev_channel_state, rpc_set_dev_mode_state,
    rpc_set_local_loopback_only, rpc_set_ssh_key_state,
};
use jetkvm_client::cloud::{
    rpc_deregister_device, rpc_get_cloud_state, rpc_get_tls_state, rpc_set_cloud_url,
    rpc_set_tls_state,
};
use jetkvm_client::console::open_console;
use jetkvm_client::device::{rpc_get_device_id, rpc_ping};
use jetkvm_client::extension::{
    rpc_get_active_extension, rpc_get_serial_settings, rpc_set_active_extension,
    rpc_set_serial_settings,
};
use jetkvm_client::hardware::{
    rpc_get_backlight_settings, rpc_get_display_rotation, rpc_set_backlight_settings,
    rpc_set_display_rotation,
};
use jetkvm_client::jiggler::{
    rpc_get_jiggler_config, rpc_get_jiggler_state, rpc_set_jiggler_config, rpc_set_jiggler_state,
};
use jetkvm_client::jetkvm_rpc_client::{JetKvmRpcClient, SignalingMethod};
use serde_json::{json, Value};
use jetkvm_client::keyboard::{
    rpc_get_key_down_state, rpc_get_keyboard_layout, rpc_get_keyboard_led_state,
    rpc_keyboard_report, rpc_sendtext, rpc_set_keyboard_layout, send_ctrl_a, send_ctrl_c,
    send_ctrl_v, send_ctrl_x, send_key_combinations, send_return, send_text_with_layout,
    send_windows_key, KeyCombo,
};
use jetkvm_client::mouse::{
    rpc_abs_mouse_report, rpc_double_click, rpc_left_click, rpc_left_click_and_drag_to_center,
    rpc_middle_click, rpc_move_mouse, rpc_rel_mouse_report, rpc_right_click, rpc_wheel_report,
};
use jetkvm_client::network::{
    rpc_get_network_settings, rpc_get_network_state, rpc_renew_dhcp_lease,
    rpc_set_network_settings,
};
use jetkvm_client::power::{
    rpc_get_atx_state, rpc_get_dc_power_state, rpc_set_atx_power_action, rpc_set_dc_power_state,
    rpc_set_dc_restore_state,
};
use jetkvm_client::storage::{
    rpc_delete_storage_file, rpc_get_storage_space, rpc_get_virtual_media_state,
    rpc_list_storage_files, rpc_mount_with_http, rpc_mount_with_storage,
    rpc_start_storage_file_upload, rpc_unmount_image,
};
use jetkvm_client::system::{
    rpc_get_auto_update_state, rpc_get_edid, rpc_get_local_version, rpc_get_timezones,
    rpc_get_update_status, rpc_reboot, rpc_set_auto_update_state, rpc_set_edid, rpc_try_update,
};
use jetkvm_client::usb::{
    rpc_get_usb_config, rpc_get_usb_devices, rpc_get_usb_emulation_state, rpc_set_usb_config,
    rpc_set_usb_devices, rpc_set_usb_emulation_state,
};
use jetkvm_client::video::{
    rpc_get_stream_quality_factor, rpc_get_video_log_status, rpc_get_video_state,
};
use jetkvm_client::wol::{
    rpc_get_wake_on_lan_devices, rpc_send_wol_magic_packet, rpc_set_wake_on_lan_devices,
};
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
    /// Sends a sequence of key combinations (JSON format).
    #[command(name = "send-key-combinations")]
    SendKeyCombinations { combos: String },
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
    /// Clicks and drags from a position to center.
    #[command(name = "left-click-and-drag-to-center")]
    LeftClickAndDragToCenter { start_x: i64, start_y: i64 },
    /// Captures a screenshot as PNG (returns base64 encoded data URL).
    #[command(name = "screenshot")]
    Screenshot {
        #[arg(long)]
        output: Option<String>,
    },
    /// Waits for the specified number of milliseconds.
    #[command(name = "wait")]
    Wait { milliseconds: u64 },
    /// Gets the current keyboard layout.
    #[command(name = "get-keyboard-layout")]
    GetKeyboardLayout,
    /// Sets the keyboard layout.
    #[command(name = "set-keyboard-layout")]
    SetKeyboardLayout { layout: String },
    /// Gets the keyboard LED state (Caps/Num Lock).
    #[command(name = "get-keyboard-led-state")]
    GetKeyboardLedState,
    /// Gets the currently pressed keys.
    #[command(name = "get-key-down-state")]
    GetKeyDownState,
    /// Sends a relative mouse report with dx, dy and button state.
    #[command(name = "rel-mouse-report")]
    RelMouseReport { dx: i64, dy: i64, buttons: u64 },
    /// Gets the virtual media state.
    #[command(name = "get-virtual-media-state")]
    GetVirtualMediaState,
    /// Mounts virtual media from HTTP URL.
    #[command(name = "mount-with-http")]
    MountWithHttp { url: String, mode: String },
    /// Mounts virtual media from storage.
    #[command(name = "mount-with-storage")]
    MountWithStorage { filename: String, mode: String },
    /// Unmounts virtual media.
    #[command(name = "unmount-image")]
    UnmountImage,
    /// Lists files in storage.
    #[command(name = "list-storage-files")]
    ListStorageFiles,
    /// Gets available storage space.
    #[command(name = "get-storage-space")]
    GetStorageSpace,
    /// Deletes a file from storage.
    #[command(name = "delete-storage-file")]
    DeleteStorageFile { filename: String },
    /// Starts a file upload to storage.
    #[command(name = "start-storage-file-upload")]
    StartStorageFileUpload { filename: String, size: u64 },
    /// Gets network settings.
    #[command(name = "get-network-settings")]
    GetNetworkSettings,
    /// Sets network settings.
    #[command(name = "set-network-settings")]
    SetNetworkSettings { settings: String },
    /// Gets network state.
    #[command(name = "get-network-state")]
    GetNetworkState,
    /// Renews DHCP lease.
    #[command(name = "renew-dhcp-lease")]
    RenewDhcpLease,
    /// Gets ATX power state.
    #[command(name = "get-atx-state")]
    GetAtxState,
    /// Sets ATX power action (power on/off/reset).
    #[command(name = "set-atx-power-action")]
    SetAtxPowerAction { action: String },
    /// Gets DC power state.
    #[command(name = "get-dc-power-state")]
    GetDcPowerState,
    /// Sets DC power state.
    #[command(name = "set-dc-power-state")]
    SetDcPowerState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Sets DC restore state.
    #[command(name = "set-dc-restore-state")]
    SetDcRestoreState { state: u64 },
    /// Gets USB configuration.
    #[command(name = "get-usb-config")]
    GetUsbConfig,
    /// Sets USB configuration.
    #[command(name = "set-usb-config")]
    SetUsbConfig { config: String },
    /// Gets USB devices.
    #[command(name = "get-usb-devices")]
    GetUsbDevices,
    /// Sets USB devices.
    #[command(name = "set-usb-devices")]
    SetUsbDevices { devices: String },
    /// Gets USB emulation state.
    #[command(name = "get-usb-emulation-state")]
    GetUsbEmulationState,
    /// Sets USB emulation state.
    #[command(name = "set-usb-emulation-state")]
    SetUsbEmulationState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Reboots the device.
    #[command(name = "reboot")]
    Reboot {
        #[arg(long, default_value = "false")]
        force: bool,
    },
    /// Gets the local firmware version.
    #[command(name = "get-local-version")]
    GetLocalVersion,
    /// Gets the firmware update status.
    #[command(name = "get-update-status")]
    GetUpdateStatus,
    /// Attempts to update the firmware.
    #[command(name = "try-update")]
    TryUpdate,
    /// Gets the auto-update state.
    #[command(name = "get-auto-update-state")]
    GetAutoUpdateState,
    /// Sets the auto-update state.
    #[command(name = "set-auto-update-state")]
    SetAutoUpdateState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Gets the list of available timezones.
    #[command(name = "get-timezones")]
    GetTimezones,
    /// Gets the mouse jiggler state.
    #[command(name = "get-jiggler-state")]
    GetJigglerState,
    /// Sets the mouse jiggler state.
    #[command(name = "set-jiggler-state")]
    SetJigglerState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Gets the mouse jiggler configuration.
    #[command(name = "get-jiggler-config")]
    GetJigglerConfig,
    /// Sets the mouse jiggler configuration.
    #[command(name = "set-jiggler-config")]
    SetJigglerConfig { config: String },
    /// Gets the video stream state.
    #[command(name = "get-video-state")]
    GetVideoState,
    /// Gets the stream quality factor.
    #[command(name = "get-stream-quality-factor")]
    GetStreamQualityFactor,
    /// Gets the video logging status.
    #[command(name = "get-video-log-status")]
    GetVideoLogStatus,
    /// Gets Wake-on-LAN devices.
    #[command(name = "get-wake-on-lan-devices")]
    GetWakeOnLanDevices,
    /// Sets Wake-on-LAN devices.
    #[command(name = "set-wake-on-lan-devices")]
    SetWakeOnLanDevices { params: String },
    /// Sends a Wake-on-LAN magic packet.
    #[command(name = "send-wol-magic-packet")]
    SendWolMagicPacket { mac_address: String },
    /// Gets the cloud connection state.
    #[command(name = "get-cloud-state")]
    GetCloudState,
    /// Sets the cloud URL.
    #[command(name = "set-cloud-url")]
    SetCloudUrl { api_url: String, app_url: String },
    /// Gets the TLS state.
    #[command(name = "get-tls-state")]
    GetTlsState,
    /// Sets the TLS state.
    #[command(name = "set-tls-state")]
    SetTlsState {
        mode: String,
        certificate: String,
        private_key: String,
    },
    /// Deregisters the device from the cloud.
    #[command(name = "deregister-device")]
    DeregisterDevice,
    /// Gets the developer mode state.
    #[command(name = "get-dev-mode-state")]
    GetDevModeState,
    /// Sets the developer mode state.
    #[command(name = "set-dev-mode-state")]
    SetDevModeState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Gets the SSH key state.
    #[command(name = "get-ssh-key-state")]
    GetSshKeyState,
    /// Sets the SSH key.
    #[command(name = "set-ssh-key-state")]
    SetSshKeyState { ssh_key: String },
    /// Gets the dev channel state.
    #[command(name = "get-dev-channel-state")]
    GetDevChannelState,
    /// Sets the dev channel state.
    #[command(name = "set-dev-channel-state")]
    SetDevChannelState {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Gets the local loopback only setting.
    #[command(name = "get-local-loopback-only")]
    GetLocalLoopbackOnly,
    /// Sets the local loopback only setting.
    #[command(name = "set-local-loopback-only")]
    SetLocalLoopbackOnly {
        #[arg(action = ArgAction::Set)]
        enabled: bool,
    },
    /// Resets the device configuration to factory defaults.
    #[command(name = "reset-config")]
    ResetConfig,
    /// Sets the display rotation.
    #[command(name = "set-display-rotation")]
    SetDisplayRotation { rotation: String },
    /// Gets the display rotation.
    #[command(name = "get-display-rotation")]
    GetDisplayRotation,
    /// Sets the backlight settings.
    #[command(name = "set-backlight-settings")]
    SetBacklightSettings {
        max_brightness: i32,
        dim_after: i32,
        off_after: i32,
    },
    /// Gets the backlight settings.
    #[command(name = "get-backlight-settings")]
    GetBacklightSettings,
    /// Gets the active extension ID.
    #[command(name = "get-active-extension")]
    GetActiveExtension,
    /// Sets the active extension.
    #[command(name = "set-active-extension")]
    SetActiveExtension { extension_id: String },
    /// Gets the serial console settings.
    #[command(name = "get-serial-settings")]
    GetSerialSettings,
    /// Sets the serial console settings.
    #[command(name = "set-serial-settings")]
    SetSerialSettings {
        baud_rate: String,
        data_bits: String,
        stop_bits: String,
        parity: String,
    },
    /// Opens an interactive serial console.
    #[command(name = "open-console")]
    OpenConsole,
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
            Commands::SendKeyCombinations { combos } => {
                let combos_vec: Vec<KeyCombo> = serde_json::from_str(&combos)?;
                send_key_combinations(&client, combos_vec)
                    .await
                    .map(|_| json!({ "status": "ok" }))
                    .map_err(|e| anyhow::anyhow!("{}", e))
            }
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
            Commands::LeftClickAndDragToCenter { start_x, start_y } => {
                rpc_left_click_and_drag_to_center(&client, start_x, start_y)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::Screenshot { output } => {
                client.add_video_transceiver().await?;
                client.video_capture.wait_for_track().await?;
                let result = client
                    .video_capture
                    .capture_screenshot_png()
                    .await
                    .and_then(|png_data| {
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        let data_url = format!("data:image/png;base64,{}", base64_data);

                        let mut result = json!({
                            "status": "ok",
                            "format": "png",
                            "size": png_data.len(),
                            "data": data_url
                        });

                        if let Some(output_path) = output {
                            let mut file = File::create(&output_path)
                                .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
                            file.write_all(&png_data)
                                .map_err(|e| anyhow::anyhow!("Failed to write to output file: {}", e))?;
                            result["saved_to"] = json!(output_path);
                        }

                        Ok(result)
                    });
                client.remove_video_track().await?;
                result
            }
            Commands::Wait { milliseconds } => {
                tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
                Ok(json!({ "status": "ok" }))
            }
            Commands::GetKeyboardLayout => rpc_get_keyboard_layout(&client).await,
            Commands::SetKeyboardLayout { layout } => rpc_set_keyboard_layout(&client, layout)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetKeyboardLedState => rpc_get_keyboard_led_state(&client).await,
            Commands::GetKeyDownState => rpc_get_key_down_state(&client).await,
            Commands::RelMouseReport { dx, dy, buttons } => {
                rpc_rel_mouse_report(&client, dx, dy, buttons)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetVirtualMediaState => rpc_get_virtual_media_state(&client).await,
            Commands::MountWithHttp { url, mode } => rpc_mount_with_http(&client, url, mode)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::MountWithStorage { filename, mode } => {
                rpc_mount_with_storage(&client, filename, mode)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::UnmountImage => rpc_unmount_image(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::ListStorageFiles => rpc_list_storage_files(&client).await,
            Commands::GetStorageSpace => rpc_get_storage_space(&client).await,
            Commands::DeleteStorageFile { filename } => {
                rpc_delete_storage_file(&client, filename)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::StartStorageFileUpload { filename, size } => {
                rpc_start_storage_file_upload(&client, filename, size)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetNetworkSettings => rpc_get_network_settings(&client).await,
            Commands::SetNetworkSettings { settings } => {
                let settings_json: Value = serde_json::from_str(&settings)?;
                rpc_set_network_settings(&client, settings_json)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetNetworkState => rpc_get_network_state(&client).await,
            Commands::RenewDhcpLease => rpc_renew_dhcp_lease(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetAtxState => rpc_get_atx_state(&client).await,
            Commands::SetAtxPowerAction { action } => rpc_set_atx_power_action(&client, action)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetDcPowerState => rpc_get_dc_power_state(&client).await,
            Commands::SetDcPowerState { enabled } => rpc_set_dc_power_state(&client, enabled)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SetDcRestoreState { state } => rpc_set_dc_restore_state(&client, state)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetUsbConfig => rpc_get_usb_config(&client).await,
            Commands::SetUsbConfig { config } => {
                let config_json: Value = serde_json::from_str(&config)?;
                rpc_set_usb_config(&client, config_json)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetUsbDevices => rpc_get_usb_devices(&client).await,
            Commands::SetUsbDevices { devices } => {
                let devices_json: Value = serde_json::from_str(&devices)?;
                rpc_set_usb_devices(&client, devices_json)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetUsbEmulationState => rpc_get_usb_emulation_state(&client).await,
            Commands::SetUsbEmulationState { enabled } => {
                rpc_set_usb_emulation_state(&client, enabled)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::Reboot { force } => rpc_reboot(&client, force)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetLocalVersion => rpc_get_local_version(&client).await,
            Commands::GetUpdateStatus => rpc_get_update_status(&client).await,
            Commands::TryUpdate => rpc_try_update(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetAutoUpdateState => rpc_get_auto_update_state(&client).await,
            Commands::SetAutoUpdateState { enabled } => {
                rpc_set_auto_update_state(&client, enabled)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetTimezones => rpc_get_timezones(&client).await,
            Commands::GetJigglerState => rpc_get_jiggler_state(&client).await,
            Commands::SetJigglerState { enabled } => rpc_set_jiggler_state(&client, enabled)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetJigglerConfig => rpc_get_jiggler_config(&client).await,
            Commands::SetJigglerConfig { config } => {
                let config_json: Value = serde_json::from_str(&config)?;
                rpc_set_jiggler_config(&client, config_json)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetVideoState => rpc_get_video_state(&client).await,
            Commands::GetStreamQualityFactor => rpc_get_stream_quality_factor(&client).await,
            Commands::GetVideoLogStatus => rpc_get_video_log_status(&client).await,
            Commands::GetWakeOnLanDevices => rpc_get_wake_on_lan_devices(&client).await,
            Commands::SetWakeOnLanDevices { params } => {
                let params_json: Value = serde_json::from_str(&params)?;
                rpc_set_wake_on_lan_devices(&client, params_json)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::SendWolMagicPacket { mac_address } => {
                rpc_send_wol_magic_packet(&client, mac_address)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetCloudState => rpc_get_cloud_state(&client).await,
            Commands::SetCloudUrl { api_url, app_url } => rpc_set_cloud_url(&client, &api_url, &app_url)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetTlsState => rpc_get_tls_state(&client).await,
            Commands::SetTlsState {
                mode,
                certificate,
                private_key,
            } => rpc_set_tls_state(&client, &mode, &certificate, &private_key)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::DeregisterDevice => rpc_deregister_device(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetDevModeState => rpc_get_dev_mode_state(&client).await,
            Commands::SetDevModeState { enabled } => rpc_set_dev_mode_state(&client, enabled)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetSshKeyState => rpc_get_ssh_key_state(&client).await,
            Commands::SetSshKeyState { ssh_key } => rpc_set_ssh_key_state(&client, &ssh_key)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetDevChannelState => rpc_get_dev_channel_state(&client).await,
            Commands::SetDevChannelState { enabled } => {
                rpc_set_dev_channel_state(&client, enabled)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetLocalLoopbackOnly => rpc_get_local_loopback_only(&client).await,
            Commands::SetLocalLoopbackOnly { enabled } => {
                rpc_set_local_loopback_only(&client, enabled)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::ResetConfig => rpc_reset_config(&client)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::SetDisplayRotation { rotation } => {
                rpc_set_display_rotation(&client, &rotation)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetDisplayRotation => rpc_get_display_rotation(&client).await,
            Commands::SetBacklightSettings {
                max_brightness,
                dim_after,
                off_after,
            } => rpc_set_backlight_settings(&client, max_brightness, dim_after, off_after)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::GetBacklightSettings => rpc_get_backlight_settings(&client).await,
            Commands::GetActiveExtension => rpc_get_active_extension(&client).await,
            Commands::SetActiveExtension { extension_id } => {
                rpc_set_active_extension(&client, &extension_id)
                    .await
                    .map(|_| json!({ "status": "ok" }))
            }
            Commands::GetSerialSettings => rpc_get_serial_settings(&client).await,
            Commands::SetSerialSettings {
                baud_rate,
                data_bits,
                stop_bits,
                parity,
            } => rpc_set_serial_settings(&client, &baud_rate, &data_bits, &stop_bits, &parity)
                .await
                .map(|_| json!({ "status": "ok" })),
            Commands::OpenConsole => {
                let serial_channel = client.create_serial_channel().await?;
                open_console(serial_channel).await
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
