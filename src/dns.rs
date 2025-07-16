use hickory_proto::rr::LowerName;
use hickory_server::authority::{Catalog, ZoneType};
use hickory_server::store::in_memory::InMemoryAuthority;
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo, ServerFuture};
use hickory_proto::rr::{Name, RData, Record};
use tonic::async_trait;
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::RwLock;

/* 
#[derive(Clone)]
pub struct SharedCatalog(pub Arc<RwLock<Catalog>>);

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
*/

#[derive(Clone)]
pub struct SharedCatalog(pub Arc<RwLock<Catalog>>);

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

pub struct DnsState {
    catalog: Arc<RwLock<Catalog>>,
    authority: Arc<InMemoryAuthority>,
    origin: LowerName,
}

impl DnsState {
    pub fn new() -> anyhow::Result<Self> {
        let origin = LowerName::new(&Name::from_ascii("example.com.")?);
        let authority = Arc::new(InMemoryAuthority::empty(origin.clone().into(), ZoneType::Primary, false));

        let mut catalog = Catalog::new();
        catalog.upsert(origin.clone(), Box::new(authority.clone()));

        Ok(Self {
            catalog: Arc::new(RwLock::new(catalog)),
            authority,
            origin,
        })
    }

    pub async fn add_record(&self, name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        let fqdn = Name::from_ascii(&name)?;
        let ip = value.parse()?;
        let record = Record::from_rdata(fqdn, ttl, RData::A(ip));
        self.authority.upsert(record, 0).await;
        Ok(())
    }

    pub fn catalog(&self) -> Arc<RwLock<Catalog>> {
        self.catalog.clone()
    }
}

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