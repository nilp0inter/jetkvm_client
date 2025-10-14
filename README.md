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
  
## About the cmdline client

The client (cargo run/jetkvm_client) is a simple client that connects to the JetKVM device, performs a few RPC calls to get device information, and then disconnects.

```
A client for JetKVM over WebRTC.

Usage: jetkvm_client [OPTIONS]

Options:
  -H, --host <HOST>          The host address to connect to
  -p, --port <PORT>          The port number to use
  -a, --api <API>            The API endpoint
  -P, --password <PASSWORD>  The password for authentication
  -v, --verbose              Enable verbose logging (include logs from webrtc_sctp)
  -h, --help                 Print help
  -V, --version              Print version
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
