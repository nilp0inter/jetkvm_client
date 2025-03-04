# jetkvm_control

**jetkvm_control** is a Rust library and client for interacting with JetKVM devices using WebRTC and JSON‑RPC. It provides functionality to authenticate with a JetKVM server, set up a WebRTC PeerConnection with a DataChannel, and send various input events (keyboard and mouse) as well as receive notifications (such as screen resolution updates) from the device.

## Features

- **HTTP Authentication:** Log in to a JetKVM server with cookie-based authentication.
- **WebRTC Connection:** Establish a WebRTC PeerConnection and DataChannel.
- **JSON‑RPC Messaging:** Send JSON‑RPC calls over the DataChannel for various commands.
- **Keyboard Input:** Functions for sending keyboard events including text, control combinations (Ctrl-A, Ctrl-C, Ctrl-V, Ctrl-X, etc.), and special keys (Return, Windows key, etc.).
- **Mouse Control:** Functions for absolute mouse movement, clicks (left, right, middle), double-click, and click-and-drag actions.
- **Notification Handling:** Receive notifications (e.g. videoInputState) and update internal state (such as screen resolution).
- **Configurable:** Easily configure connection parameters (IP, port, API endpoint, and password) using environment variables. For a simple parameter bag, use the tuple-struct `JetKvmParams`.

## Installation

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
jetkvm_control = "0.1.3"  # or use a git dependency / local path during development
```

### Setup

1. **Install via cargo**
   ```
   cargo install jetkvm_control
   ```

   - or - 

   **Clone the Repository**
      ```
      git clone https://github.com/davehorner/jetkvm_control.git
      cd jetkvm_control
      ```

3. **Configure Your Settings**

    Before running the project, update your configuration settings. The project reads its configuration from either a config.toml file or environment variables. For example, create a config.toml with your settings:
      ```toml
      host = "host/ip"
      password = "your_password_here"
      port = "80"
      api = "/webrtc/session"
      ```

    You can also override these values via the command-line. For example:
    ```
    cargo run -- --host 192.168.1.100 --port 8080
    ```

4. **Running the Project**
    After setting up your configuration, you can build and run the project with Cargo:
     ```bash
     cargo run -- -H 192.168.1.100 lua-examples/windows-notepad-helloworld.lua
     ```

    This will compile the project and execute the window-notepad-helloworld.lua example.
  
## About the cmdline client

The client (cargo run/jetkvm_control) is a simple ping if you don't have the `lua` feature enabled.

If you enable the lua feature; jetkvm_control will expect a lua file to execute.

```
A control client for JetKVM over WebRTC.

Usage: jetkvm_control [OPTIONS] <LUA_SCRIPT>

Arguments:
  <LUA_SCRIPT>  Path to the Lua script to execute

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

The api is subject to change.

example code for rust:
```rust
let config = JetKvmConfig::load()?;
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

example code in lua:
```lua
print("Executing Lua script...")
send_windows_key()
delay(550)
send_text("notepad")
send_return()
delay(250)
delay(250)
send_text("Hello World!")
send_ctrl_a()
send_ctrl_c()
delay(250)
send_ctrl_v()
delay(250)
send_ctrl_v()
send_key_combinations({
    { modifier = 0x04, keys = {0x3D}, hold = 100, wait = 1000 }, -- Alt+F4
    { modifier = 0, keys = {0x4F}, wait = 250 }, -- Right arrow
    { modifier = 0, keys = {0x28} },  -- Return (Enter)
})
```

Check out the examples folder for additional detail.

---

## **Configuration Loading Precedence**
The configuration file (`config.toml`) is loaded based on the following priority order:

### **📌 Priority Order**
| Priority | macOS/Linux                  | Windows                                  |
|----------|------------------------------|------------------------------------------|
| 1️⃣ **(Highest)** | `config.toml` (Current Directory) | `config.toml` (Current Directory) |
| 2️⃣ | `${CARGO_MANIFEST_DIR}/config.toml` | `${CARGO_MANIFEST_DIR}/config.toml` |
| 3️⃣ **(System-Wide)** | `/etc/jetkvm_control/config.toml` | `%APPDATA%\jetkvm_control\config.toml` |

### **📍 How Configuration is Resolved**
- **Current Directory (`config.toml`)** – Preferred for local development.
- **Cargo Project Root (`CARGO_MANIFEST_DIR/config.toml`)** – Used when running inside a Rust project.
- **System-Wide Location (`/etc/jetkvm_control/config.toml` or `%APPDATA%\jetkvm_control\config.toml`)** – Used when no local config is found.

If no configuration file is found, the program exits with an error message.

## Note
  - Password-less and Password-based local authentication have been tested functional.
  - Cloud integration and ICE/STUN support are not implemented yet.
  - Contributions for these features are welcome!

## License
This project is licensed under the MIT License. See LICENSE for details.

## Contributing
Contributions are welcome! Please submit a pull request or open an issue to discuss changes.

