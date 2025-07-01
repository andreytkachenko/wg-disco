use super::{Cidr, Endpoint, Key};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WgPeerInfo {
    // PublicKey
    pub public_key: Key,

    // PresharedKey
    pub preshared_key: Option<Key>,

    // Endpoint
    pub endpoint: Option<Endpoint>,

    // AllowedIPs
    pub allowed_ips: Option<Vec<Cidr>>,

    // PersistentKeepalive
    pub persistent_keepalive: Option<u32>,

    // LatestHandshake
    pub latest_handshake: Option<u32>,

    // Transfer
    pub transfer: Option<(u64, u64)>,
}
