# JetKVM Control Secure RPC Server

This contains the JetKVM Control Secure RPC server, designed to handle requests for system information on a host machine. The server is built using Rust and provides TLS encryption and HMAC authentication to ensure secure and authenticated communication.

JetKVM excels at remote mouse and keyboard control, while the jetkvm_control_svr complements this by enabling secure queries for the active process or window on the host machine. This added monitoring capability ensures that your scripts can verify the system’s current state before executing control actions, improving reliability and confidence in automation.

## Features

- **Cross-Platform Compatibility**: Provides access to information about the active process and window on Windows and macOS.
- **TLS Encryption**: Ensures that all communications with the server are securely encrypted using Rustls.
- **HMAC Authentication**: Verifies client requests using HMAC-SHA256 based on a shared password and a dynamically generated challenge.
- **Extensible Architecture**: Facilitates secure retrieval and response handling for various system-related commands.

## Installation

   ```bash
   git clone https://github.com/davehorner/jetkvm_control.git
   cd jetkvm_control
   cargo install --path jetkvm_control_svr 
   ```

2. Build the project:
   ```bash
   cargo build -p jetkvm_control_svr --release 
   ```

### Generate TLS Certificates

The sever can generate a self-signed certificate if you do not have a certificate already.
If you need to generate self-signed certificates:

```bash
   cargo run -p jetkvm_control_svr --release -P <password> -I
```

This will generate `cert.pem` and `key.pem` in the current directory.  A password and a certificate are required.

## Usage

Run the server with the desired configuration options. For example:

### Command Line Arguments

- `-H, --host`: Host address to bind to (default: `0.0.0.0`)
- `-p, --port`: Port to listen on (default: `8080`)
- `-P, --password`: Password for authentication (required)
- `-C, --cert-path`: Path to TLS certificate (default: `cert.pem`)
- `-K, --key-path`: Path to TLS private key (default: `key.pem`)
- `-I, --init-cert`: Initialize self-signed certificate

## Supported Commands

- `active_process`: Retrieve information about the active process.
- `active_window`: Retrieve information about the active window.

More commands can be added as needed by extending the server's request handler logic.

## Security

The server uses TLS for secure communication and HMAC for request authentication, preventing unauthorized access and ensuring data integrity.

## Contributing

Please feel free to submit issues or pull requests. Contributions are welcome!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
