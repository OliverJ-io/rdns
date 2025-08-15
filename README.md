# Rust DNS Server with gRPC Control Interface

This project implements a lightweight DNS server in Rust using [`hickory-server`](https://docs.rs/hickory-server) and exposes a gRPC API using [`tonic`](https://docs.rs/tonic). It allows dynamic management of DNS A records via RPC calls, enabling integrations or external controllers to modify DNS state at runtime.

## 🚀 Features

- 🧠 In-memory authoritative DNS server
- 🌐 gRPC control interface for dynamic record updates
- ✅ Support for adding and deleting A records
- 📦 Thread-safe shared state using `tokio::RwLock`
- 🔧 Modular structure for DNS and control layers
- 🧪 Built-in support for testing with tools like Insomnia or `grpcurl`

## 🛠 Requirements

- **Rust** (latest stable recommended)
- **Protobuf Compiler (`protoc`)**

### Install `protoc`

**On macOS:**
```sh
brew install protobuf
```

## 📁 Project Structure

```.
├── main.rs              # Entry point, starts DNS + gRPC servers
├── dns.rs               # In-memory DNS state and server logic
├── control.rs           # gRPC server + request handlers
├── proto/control.proto  # gRPC interface definition
├── build.rs             # Protobuf compilation
└── README.md            # This file
```