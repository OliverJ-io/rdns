//! gRPC control interface for managing DNS records.
//!
//! Provides a `DnsControl` gRPC server implementation that allows adding and
//! deleting DNS records in a shared `DnsState` through protobuf requests.

use tonic::{transport::Server, Request, Response, Status};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

use crate::{control::dns_control_server::DnsControl, dns::DnsState, settings::GrpcSettings};

// Generated protobuf code for the `control` service.
// Includes the request/response types and the DnsControl trait.
tonic::include_proto!("control");

/// The gRPC control server that exposes methods for managing DNS records.
pub struct ControlServer {
    /// Shared mutable access to the DNS state.
    state: Arc<RwLock<DnsState>>,
}

impl ControlServer {
    /// Constructs a new `ControlServer` with shared DNS state
    pub fn new(state: Arc<RwLock<DnsState>>) -> Self {
        Self { state }
    }
}

/// Config options for the Grpc Control Server
pub struct GrpcOptions {
    pub listen_addr: String,
}
impl From<GrpcSettings> for GrpcOptions {
    fn from(cfg: GrpcSettings) -> Self {
        GrpcOptions {
            listen_addr: cfg.listen_addr,
        }
    }
}

#[tonic::async_trait]
impl DnsControl for ControlServer {
    /// Adds a new A record to the DNS authority.
    ///
    /// This method is invoked via gRPC with a `AddRecordRequest` and returns
    /// a `ControlResponse` indicating success or failure.
    async fn add_record(
        &self,
        request: Request<AddRecordRequest>,
    ) -> Result<Response<ControlResponse>, Status> {
        // Extract request data
        let req = request.into_inner();
        // Obtain write lock on DNS state to allow mutation
        let state = self.state.write().await;
        // Attempt to add the record
        match state.add_record(req.name, req.value, req.ttl).await {
            Ok(_) => Ok(Response::new(ControlResponse {
                success: true,
                message: "Record added".into(),
            })),
            Err(e) => Ok(Response::new(ControlResponse {
                success: false,
                message: format!("Error: {}", e),
            })),
        }
    }

    /// Deletes an A record from the DNS authority.
    ///
    /// This method is invoked via gRPC with a `DeleteRecordRequest` and returns
    /// a `ControlResponse` indicating success or failure.
    async fn delete_record(
        &self,
        request: Request<DeleteRecordRequest>, 
    ) -> Result<Response<ControlResponse>, Status> {
        // Extract request data
        let req = request.into_inner();
        // Obtain write lock on DNS state to allow mutation
        let state = self.state.write().await;
        // Attempt to delete the record
        match state.delete_record(req.name).await {
            Ok(_) => Ok(Response::new(ControlResponse {
                success: true,
                message: "Record deleted".into(),
            })),
            Err(e) => Ok(Response::new(ControlResponse {
                success: false,
                message: format!("Error: {}", e),
            })),
        }
    }

    async fn get_all_records(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetAllRecordsResponse>, Status> {
        let state = self.state.read().await;
        let records = state.get_all_records().await;

        let proto_records = records
            .into_iter()
            .map(|(name, value, ttl)| DnsRecord { name, value, ttl })
            .collect();

        Ok(Response::new(GetAllRecordsResponse {
            records: proto_records,
        }))
    }
}

pub async fn run_grpc_server(service: ControlServer, options: GrpcOptions) -> anyhow::Result<()> {
    let addr: SocketAddr = options.listen_addr.parse()?;
    println!("gRPC server listening on {}", addr);
    Server::builder()
        .add_service(dns_control_server::DnsControlServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
