use std::net::{SocketAddr, ToSocketAddrs};

use stunclient::StunClient;

use super::Discover;

const DEFAULT_STUN_SERVER: &str = "stun.l.google.com:19302";

#[derive(Debug, Clone)]
pub struct StunDiscover {
    server: SocketAddr,
}

impl Default for StunDiscover {
    fn default() -> Self {
        Self::new(std::env::var("STUN_SERVER").unwrap_or_else(|_| DEFAULT_STUN_SERVER.into()))
    }
}

impl StunDiscover {
    pub fn new(server: String) -> Self {
        let server = server
            .to_socket_addrs()
            .unwrap()
            .filter(|x| x.is_ipv4())
            .next()
            .unwrap();

        Self { server }
    }
}

impl Discover for StunDiscover {
    type Error = stunclient::Error;

    async fn discover(&self) -> Result<(SocketAddr, u16), Self::Error> {
        let udp = tokio::net::UdpSocket::bind("0:0")
            .await
            .map_err(stunclient::Error::Socket)?;

        let local_port = udp.local_addr().map_err(stunclient::Error::Socket)?.port();

        let stun_client = StunClient::new(self.server);
        let addr = stun_client.query_external_address_async(&udp).await?;

        Ok((addr, local_port))
    }
}
