# jetkvm_client

**jetkvm_client** is a Rust client library for interacting with JetKVM devices using WebRTC and JSON‑RPC. It provides functionality to authenticate with a JetKVM device, set up a WebRTC PeerConnection with a DataChannel, and send various input events (keyboard and mouse) as well as receive notifications (such as screen resolution updates) from the device.

This is a fork of the original [jetkvm_control](https://github.com/davehorner/jetkvm_control) by David Horner. Thank you for your work!

## New Goals

The goal of this library is to be able to do programatically whatever a jetkvm user can do via the web interface.

## TODO

- [x] Screen capture (screenshot)
- [ ] Video recording from WebRTC stream

## Features

- **Keyboard Input:** Functions for sending keyboard events including text, control combinations (Ctrl-A, Ctrl-C, Ctrl-V, Ctrl-X, etc.), and special keys (Return, Windows key, etc.).
- **Mouse Control:** Functions for absolute mouse movement, clicks (left, right, middle), double-click, and click-and-drag actions.

## Installation

1. **Install via cargo**
   ```
   cargo install jetkvm_client
   ```

   - or - 

   **Clone the Repository**
      ```
      git clone https://github.com/nilp0inter/jetkvm_client.git
      cd jetkvm_client
      ```

2. **Running the Project**
    You can build and run the project with Cargo:
     ```bash
     cargo run -- -H 192.168.1.100 -P mypassword
     ```
  
## Usage

The `jetkvm_client` executable provides a powerful command-line interface for interacting with your JetKVM device. It uses a subcommand-based system, and you can chain multiple commands together in a single execution, similar to `xdotool`.

All output is a JSON Lines stream, where each line is a JSON object representing the result of a command. This makes it easy to parse the output in scripts.

### Basic Syntax

```bash
jetkvm_client [GLOBAL_OPTIONS] COMMAND_1 [ARGS] COMMAND_2 [ARGS] ...
```

### Global Options

These options control the connection to the JetKVM device and must be provided before any commands.

- `-H, --host <HOST>`: The host address of the JetKVM device.
- `-P, --password <PASSWORD>`: The password for authentication.
- `-p, --port <PORT>`: The port number to use (default: 80).
- `-a, --api <API>`: The API endpoint (default: /webrtc/session).
- `-v, --verbose`: Enable verbose logging.

### Examples

**Get the device ID:**

```bash
cargo run -- -H 10.4.1.194 -P password get-device-id
```

**Output:**

```json
{"result":"JTD22510012"}
```

**Chain multiple commands:**

This example gets the device ID and then sends a ping.

```bash
cargo run -- -H 10.4.1.194 -P password get-device-id ping
```

**Output:**

```json
{"result":"JTD22510012"}
{"result":{"jsonrpc":"2.0","result":{},"id":1}}
```

**Send text to the remote machine:**

```bash
cargo run -- -H 10.4.1.194 -P password sendtext "Hello, JetKVM!"
```

**Output:**
```json
{"result":{"status":"ok"}}
```

## What's the code look like

The api is subject to change.   This project adheres to the "Semantic Versioning" standard.

example code for rust:
```rust
let config = JetKvmConfig {
    host: "192.168.1.100".to_string(),
    port: "80".to_string(),
    api: "/webrtc/session".to_string(),
    password: "mypassword".to_string(),
    ca_cert_path: "cert.pem".to_string(),
    no_auto_logout: false,
};
let mut client = JetKvmRpcClient::new(config);

if let Err(err) = client.connect().await {
    error!("Failed to connect to RPC server: {:?}", err);
    std::process::exit(1);
}
// open notepad and say Hello World, copy and paste.
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
sleep(Duration::from_millis(100)).await;
send_ctrl_v(&client).await.ok();
```

## Contributions

- 5/1/25 - [Senator3223/JetKey](https://github.com/Senator3223/JetKey/)  - use python to control your JetKVM using an api very similiar to jetkvm_client.

## License
This project is licensed under the MIT License. See LICENSE for details.

## Contributing
Contributions are welcome! Please submit a pull request or open an issue to discuss changes.

## Features

- **Keyboard Control**: Send keystrokes and text to the remote system
- **Mouse Control**: Move the mouse cursor and simulate clicks
- **Video Capture**: Capture screenshots from the WebRTC video feed
- **System Management**: Get device information and configure EDID settings

## Screenshot Capture

The library now supports capturing screenshots from the WebRTC video stream:

### Using the CLI

```bash
# Capture a screenshot (uses default or detected resolution)
jetkvm_client -H 192.168.1.100:80 -P mypassword screenshot --output screenshot.png

# Capture with specific resolution
jetkvm_client -H 192.168.1.100:80 -P mypassword screenshot --output screenshot.png --width 1920 --height 1080
```

### Using the Example

```bash
# Run the screenshot example
cargo run --example screenshot -- 192.168.1.100:80 mypassword screenshot.png
```

### Using as a Library

```rust
use jetkvm_client::jetkvm_rpc_client::{JetKvmRpcClient, SignalingMethod};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = JetKvmRpcClient::new(
        "192.168.1.100:80".to_string(),
        "mypassword".to_string(),
        "/webrtc/session".to_string(),
        false,
        SignalingMethod::Auto,
    );

    client.connect().await?;
    client.wait_for_channel_open().await?;

    // Wait for video track to be established
    sleep(Duration::from_secs(2)).await;

    // Capture screenshot
    let (width, height) = {
        let size = client.screen_size.lock().await;
        size.unwrap_or((1920, 1080))
    };

    client.video_capture
        .save_screenshot_as_png("screenshot.png", width, height)
        .await?;

    client.shutdown().await;
    Ok(())
}
```

## How It Works

The screenshot functionality works by:

1. Establishing a WebRTC connection with the JetKVM device
2. Adding a video transceiver to receive the video stream
3. Capturing RTP packets from the video track
4. Decoding the raw frame data into PNG images

The video feed is received through WebRTC's media stream, similar to how the TypeScript web client displays video.
