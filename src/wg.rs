use base64::prelude::*;
use bincode::{Decode, Encode};
use config::ParseError;
use instance::WgInterfaceInfo;
use peer::WgPeerInfo;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

pub mod cmd;
pub mod config;
pub mod instance;
pub mod peer;

pub type DecodeError = base64::DecodeSliceError;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Key([u8; 32]);

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for Key {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut inner = [0u8; 32];
        BASE64_STANDARD.decode_slice(s, &mut inner)?;
        Ok(Key(inner))
    }
}

impl Key {
    pub fn random() -> Key {
        Key(rand::random())
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", BASE64_STANDARD.encode(&self.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Peer(pub Key, pub SocketAddr);

impl FromStr for Peer {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key_str, addr_str) = s.split_once(' ').ok_or(ParseError::PeerParseError)?;
        Ok(Peer(key_str.parse()?, addr_str.parse()?))
    }
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct Cidr {
    pub ip: IpAddr,
    pub mask: u8,
}

impl Default for Cidr {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            mask: 0,
        }
    }
}

impl FromStr for Cidr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip, mask) = s.split_once('/').unwrap_or((s, ""));
        let ip = ip.trim();
        let mask = mask.trim();

        let mask: u32 = if !mask.is_empty() { mask.parse()? } else { 32 };

        Ok(Cidr {
            ip: ip.parse()?,
            mask: mask as _,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Domain(String),
    Ip(SocketAddr),
}

impl From<String> for Endpoint {
    fn from(v: String) -> Self {
        Self::Domain(v)
    }
}

impl From<SocketAddr> for Endpoint {
    fn from(v: SocketAddr) -> Self {
        Self::Ip(v)
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::Domain(dom) => write!(f, "{dom}"),
            Endpoint::Ip(addr) => write!(f, "{addr}"),
        }
    }
}

impl FromStr for Endpoint {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Ok(addr) = s.parse() {
            Self::Ip(addr)
        } else {
            Self::Domain(s.to_string())
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WgState {
    pub interface: WgInterfaceInfo,
    pub peers: Vec<WgPeerInfo>,
}

pub trait WireguardApi {
    type Error;

    fn get_pub_key(&self, iface: &str) -> Result<Key, Self::Error>;
    fn get_listen_port(&self, iface: &str) -> Result<u16, Self::Error>;
    fn get_endpoints(
        &self,
        iface: &str,
    ) -> Result<std::collections::HashMap<super::Key, Option<SocketAddr>>, Self::Error>;

    fn set_listen_port(&mut self, iface: &str, port: u16) -> Result<(), Self::Error>;
    fn set_peer_endpoint(
        &mut self,
        iface: &str,
        peer: Key,
        endpoint: Endpoint,
    ) -> Result<(), Self::Error>;
}
