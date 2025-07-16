mod control;
mod dns;

use control::ControlServer;
use dns::DnsState;
use tonic::transport::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dns_state = Arc::new(RwLock::new(DnsState::new()?));

    // Spawn DNS server
    {
        let dns_state = dns_state.clone();
        println!("Starting DNS server...");
        tokio::spawn(async move {
            dns::run_dns_server(dns_state).await.unwrap();
        });
    }

    // Spawn gRPC server
    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
    let grpc_service = ControlServer::new(dns_state);

    println!("gRPC server listening on {}", grpc_addr);
    Server::builder()
        .add_service(control::dns_control_server::DnsControlServer::new(grpc_service))
        .serve(grpc_addr)
        .await?;

    Ok(())
}