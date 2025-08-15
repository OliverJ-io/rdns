use tonic::{Request, Response, Status};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{control::dns_control_server::DnsControl, dns::DnsState};

tonic::include_proto!("control");

pub struct ControlServer {
    state: Arc<RwLock<DnsState>>,
}

impl ControlServer {
    pub fn new(state: Arc<RwLock<DnsState>>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl DnsControl for ControlServer {
    async fn add_record(
        &self,
        request: Request<AddRecordRequest>,
    ) -> Result<Response<ControlResponse>, Status> {
        let req = request.into_inner();
        let state = self.state.write().await;
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
}