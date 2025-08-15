//! Application entry point for the DNS and gRPC servers.
//!
//! This binary initializes shared DNS state and concurrently runs:
//! - A DNS UDP server using `hickory-server` on port 8053
//! - A gRPC server using `tonic` on port 50051 to expose DNS management APIs via protobuf.
//!
//! The DNS server is managed by `dns::DnsState`, and the gRPC server by `control::ControlServer`.

mod control;
mod dns;

use control::ControlServer;
use dns::DnsState;
use tonic::transport::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main entry point. Initializes shared state and starts both DNS and gRPC servers.
///
/// # Returns
///
/// `anyhow::Result<()>` indicating success or failure.
///
/// # Behavior
///
/// - Initializes an `Arc<RwLock<DnsState>>` to be shared between both servers.
/// - Spawns the DNS server in a background task.
/// - Starts the gRPC server on port 50051 and blocks the main thread.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize shared DNS state with in-memory authority
    let dns_state = Arc::new(RwLock::new(DnsState::new()?));

    // Spawn the DNS server in a background task
    {
        let dns_state = dns_state.clone();
        println!("Starting DNS server...");
        tokio::spawn(async move {
            dns::run_dns_server(dns_state).await.unwrap();
        });
    }

    // Configure and start the gRPC server for DNS control
    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
    let grpc_service = ControlServer::new(dns_state);

    println!("gRPC server listening on {}", grpc_addr);
    Server::builder()
        .add_service(control::dns_control_server::DnsControlServer::new(grpc_service))
        .serve(grpc_addr)
        .await?;

    Ok(())
}