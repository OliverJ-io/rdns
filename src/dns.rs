//! DNS Server Module
//!
//! This module sets up an in-memory authoritative DNS server using the `hickory-server` crate.
//! It defines `DnsState` for managing DNS records, provides an async implementation of the
//! `RequestHandler` trait to handle DNS requests, and exposes functions to add/delete records
//! via the `InMemoryAuthority`. It also provides a `run_dns_server` function to start the UDP server.

use hickory_proto::rr::{LowerName, RrKey};
use hickory_server::authority::{Catalog, ZoneType};
use hickory_server::store::in_memory::InMemoryAuthority;
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo, ServerFuture};
use hickory_proto::rr::{Name, RData, Record, RecordType};
use tonic::async_trait;
use std::net::UdpSocket;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Wrapper around a shared, asynchronously accessible DNS catalog.
#[derive(Clone)]
pub struct SharedCatalog(pub Arc<RwLock<Catalog>>);

/// Implements DNS request handling by delegating to the inner shared catalog.
#[async_trait]
impl RequestHandler for SharedCatalog {
    async fn handle_request<R>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo
    where
        R: ResponseHandler + Send,
    {
        let catalog = self.0.read().await;
        catalog.handle_request(request, response_handle).await
    }
}

/// Holds the state of the DNS server, including the authoritative data and catalog.
pub struct DnsState {
    catalog: Arc<RwLock<Catalog>>,
    authority: Arc<InMemoryAuthority>,
    // origin: LowerName,
}

impl DnsState {
    /// Constructs a new `DnsState` with an empty authoritative zone for `example.com.
    pub fn new() -> anyhow::Result<Self> {
        let origin = LowerName::new(&Name::from_ascii("example.com.")?);
        let authority = Arc::new(InMemoryAuthority::empty(origin.clone().into(), ZoneType::Primary, false));

        let mut catalog = Catalog::new();
        catalog.upsert(origin.clone(), Box::new(authority.clone()));

        Ok(Self {
            catalog: Arc::new(RwLock::new(catalog)),
            authority,
        })
    }

    /// Helper function to construct an A record from input fields.
    fn build_a_record(name: String, value: String, ttl: u32) -> anyhow::Result<Record> {
        let fqdn = Name::from_ascii(&name)?;
        let ip = value.parse()?;
        let record = Record::from_rdata(fqdn, ttl, RData::A(ip));
        Ok(record)
    }

    /// Helper function to construct an RrKey for name record mutation
    fn build_a_record_key(name: String) -> anyhow::Result<RrKey,anyhow::Error> {
        let name = LowerName::from_str(&name)?;
        let rr_key = RrKey::new(name, RecordType::A);
        Ok(rr_key)
    }

    /// Adds an A record to the in-memory DNS zone.
    pub async fn add_record(&self, name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        let record = DnsState::build_a_record(name, value, ttl)?;
        self.authority.upsert(record, 0).await;
        Ok(())
    }

    /// Deletes an A record (by key) from the in-memory DNS zone.
    pub async fn delete_record(&self, name: String) -> anyhow::Result<()> {
        let key = DnsState::build_a_record_key(name)?;
        let mut records = self.authority.records_mut().await;
        records.remove(&key);
        Ok(())
    }

    /// Returns a clone of the internal DNS catalog reference.
    pub fn catalog(&self) -> Arc<RwLock<Catalog>> {
        self.catalog.clone()
    }
}


/// Starts the DNS server on UDP port 8053 using the provided `DnsState`.
///
/// Binds a UDP socket, wraps it in a `tokio::net::UdpSocket`, and launches
/// the `ServerFuture` from the hickory-server crate to handle requests.
///
/// # Errors
///
/// Returns an error if the socket binding, conversion, or server execution fails.
pub async fn run_dns_server(state: Arc<RwLock<DnsState>>) -> anyhow::Result<()> {
    let std_socket = UdpSocket::bind("0.0.0.0:8053")?;
    std_socket.set_nonblocking(true)?;
    let tokio_socket = tokio::net::UdpSocket::from_std(std_socket)?;

    let catalog = {
        let state = state.read().await;
        state.catalog() // Arc<RwLock<Catalog>>
    };

    let handler = SharedCatalog(catalog);
    let mut server = ServerFuture::new(handler);
    server.register_socket(tokio_socket);

    println!("DNS server listening on 0.0.0.0:8053 (UDP)");
    server.block_until_done().await?;
    Ok(())
}