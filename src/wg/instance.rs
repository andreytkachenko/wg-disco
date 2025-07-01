use std::net::IpAddr;

use super::{Cidr, Key};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WgInterfaceInfo {
    // PrivateKey
    pub private_key: Key,

    // PrivateKey
    pub public_key: Option<Key>,

    // Address
    pub address: Cidr,

    // ListenPort
    pub listen_port: Option<u16>,

    // MTU
    pub mtu: Option<u16>,

    // DNS
    pub dns: Option<Vec<IpAddr>>,

    // Table
    pub table: Option<u32>,

    // Table
    pub fwmark: Option<u32>,
}
