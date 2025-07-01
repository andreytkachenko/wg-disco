use std::net::SocketAddr;

pub mod stun;

pub mod fake {
    #[derive(Debug)]
    pub enum Void {}
    use std::net::{SocketAddr, SocketAddrV4};

    use super::Discover;

    #[derive(Debug, Default)]
    pub struct FakeDiscover;
    impl Discover for FakeDiscover {
        type Error = Void;

        async fn discover(&self) -> Result<(std::net::SocketAddr, u16), Self::Error> {
            Ok((
                SocketAddr::V4(SocketAddrV4::new([127, 0, 0, 1].into(), 51039)),
                51039,
            ))
        }
    }
}

pub trait Discover {
    type Error;
    async fn discover(&self) -> Result<(SocketAddr, u16), Self::Error>;
}
