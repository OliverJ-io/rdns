/// Defines configuration structure for Config.toml
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub dns: DnsSettings,
    pub grpc: GrpcSettings,
}

#[derive(Debug, Deserialize)]
pub struct DnsSettings {
    pub listen_addr: String,
}

#[derive(Debug, Deserialize)]
pub struct GrpcSettings {
    pub listen_addr: String,
}
