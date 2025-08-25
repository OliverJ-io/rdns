//! Application entry point for the DNS and gRPC servers.
//!
//! This binary initializes shared DNS state and concurrently runs:
//! - A DNS UDP server using `hickory-server` on port 8053
//! - A gRPC server using `tonic` on port 50051 to expose DNS management APIs via protobuf.
//!
//! The DNS server is managed by `dns::DnsState`, and the gRPC server by `control::ControlServer`.

mod control;
mod dns;
mod settings;

use settings::Settings;
use control::ControlServer;
use dns::DnsState;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::control::GrpcOptions;
use crate::dns::DnsOptions;

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
    // load config settings
    let settings = load_settings().expect("Failed to load config");
    let dns_options = DnsOptions::from(settings.dns);
    let grpc_options = GrpcOptions::from(settings.grpc);

    // Initialize shared DNS state with in-memory authority
    let dns_state = Arc::new(RwLock::new(DnsState::new()?));

    // Spawn the DNS server in a background task
    {
        let dns_state = dns_state.clone();
        tokio::spawn(async move {
            dns::run_dns_server(dns_state.clone(),dns_options).await.unwrap();
        });
    }

    control::run_grpc_server(ControlServer::new(dns_state), grpc_options).await?;

    Ok(())
}

/// load settings from the Config.toml file
fn load_settings() -> Result<Settings, config::ConfigError> {
    let builder = config::Config::builder()
        .add_source(config::File::with_name("Config").required(false))
        .add_source(config::Environment::with_prefix("APP").separator("__")); // optional
    builder.build()?.try_deserialize()
}
