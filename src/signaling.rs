use std::net::SocketAddr;

use bincode::{Decode, Encode};
use futures::Stream;
use irc::PeerEvent;

use crate::wg::{Cidr, Key};

pub mod irc;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct PeerUpdate {
    pub key: Key,
    pub endpoint: SocketAddr,
    pub advertise_routes: Vec<Cidr>,
}

// Register
pub trait Signaling {
    type Error;

    async fn announce(&mut self, peer: PeerUpdate, nick: Option<&str>) -> Result<(), Self::Error>;
    async fn subscribe(
        &mut self,
    ) -> Result<impl Stream<Item = Result<PeerEvent, Self::Error>>, Self::Error>;
}
