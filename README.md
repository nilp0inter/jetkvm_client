# jetkvm_client

**jetkvm_client** is a Rust client library for interacting with JetKVM devices using WebRTC and JSON‑RPC. It provides functionality to authenticate with a JetKVM device, set up a WebRTC PeerConnection with a DataChannel, and send various input events (keyboard and mouse) as well as receive notifications (such as screen resolution updates) from the device.

This is a fork of the original [jetkvm_control](https://github.com/davehorner/jetkvm_control) by David Horner. Thank you for your work!

## New Goals

The goal of this library is to be able to do programatically whatever a jetkvm user can do via the web interface.

## TODO

- [ ] Screen capture (screenshot and video)

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
