use std::{
    net::{AddrParseError, IpAddr},
    num::ParseIntError,
    result::Result,
    str::FromStr,
};

use super::{Cidr, DecodeError, Endpoint, Key, instance::WgInterfaceInfo, peer::WgPeerInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgConfig {
    pub interface: WgConfigInterface,
    pub peers: Vec<WgConfigPeer>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WgConfigInterface {
    // PrivateKey
    pub private_key: Key,

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

    // Instance Information
    pub advertise_routes: Option<Vec<Cidr>>,

    // PreUp
    pub pre_up: Option<String>,

    // PreDown
    pub pre_down: Option<String>,

    // PostUp
    pub post_up: Option<String>,

    // PostDown
    pub post_down: Option<String>,

    // SaveConfig
    pub save_config: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WgConfigPeer {
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
}

impl From<WgConfigPeer> for WgPeerInfo {
    fn from(peer: WgConfigPeer) -> Self {
        WgPeerInfo {
            public_key: peer.public_key,
            preshared_key: peer.preshared_key,
            endpoint: peer.endpoint,
            allowed_ips: peer.allowed_ips,
            persistent_keepalive: peer.persistent_keepalive,
            latest_handshake: None,
            transfer: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token")]
    UnexpectedToken,

    #[error("key parse error: {0}")]
    KeyParseError(#[from] DecodeError),

    #[error("addr parse error: {0}")]
    SocketAddrParseError(#[from] AddrParseError),

    #[error("expected char: {0}")]
    Expected(char),

    #[error("int parse error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("no interface section")]
    NoIntrerfaceSection,

    #[error("wrong peer format")]
    PeerParseError,
}
impl WgConfigInterface {
    fn parse(input: &mut &str) -> Result<Self, ParseError> {
        if !input.starts_with("[Interface]") {
            return Err(ParseError::UnexpectedToken);
        }

        let _ = until::<Stub>('\n', input);

        let mut iface = WgConfigInterface::default();
        while !input.is_empty() && !input.trim_start().starts_with("[Peer]") {
            match until('=', input)? {
                WgPropKind::PrivateKey => iface.private_key = until('\n', input)?,
                WgPropKind::Address => iface.address = until('\n', input)?,
                WgPropKind::ListenPort => iface.listen_port = Some(until('\n', input)?),
                WgPropKind::FWMark => iface.listen_port = Some(until('\n', input)?),
                WgPropKind::MTU => iface.mtu = Some(until('\n', input)?),
                WgPropKind::DNS => iface.dns = Some(until::<List<IpAddr>>('\n', input)?.0),
                WgPropKind::Table => iface.table = Some(until('\n', input)?),
                WgPropKind::AdvertiseRoutes => {
                    iface.advertise_routes = Some(until::<List<Cidr>>('\n', input)?.0)
                }
                WgPropKind::PostUp => iface.post_up = Some(until::<Str>('\n', input)?.0),
                WgPropKind::PostDown => iface.post_down = Some(until::<Str>('\n', input)?.0),
                WgPropKind::PreUp => iface.pre_up = Some(until::<Str>('\n', input)?.0),
                WgPropKind::PreDown => iface.pre_down = Some(until::<Str>('\n', input)?.0),
                _ => _ = until::<Stub>('\n', input)?,
            }
        }

        Ok(iface)
    }
}

impl WgConfigPeer {
    fn parse(input: &mut &str) -> Result<Self, ParseError> {
        if !input.trim_start().starts_with("[Peer]") {
            return Err(ParseError::UnexpectedToken);
        }

        let _ = until::<Stub>('\n', input);

        let mut peer = WgConfigPeer::default();
        while !input.is_empty() && !input.trim_start().starts_with("[Peer]") {
            match until('=', input)? {
                WgPropKind::PublicKey => peer.public_key = until('\n', input)?,
                WgPropKind::PresharedKey => peer.preshared_key = Some(until('\n', input)?),
                WgPropKind::Endpoint => peer.endpoint = Some(until('\n', input)?),
                WgPropKind::AllowedIPs => {
                    peer.allowed_ips = Some(until::<List<Cidr>>('\n', input)?.0)
                }
                WgPropKind::PersistentKeepalive => {
                    peer.persistent_keepalive = Some(until('\n', input)?)
                }
                _ => _ = until::<Stub>('\n', input)?,
            }
        }

        Ok(peer)
    }
}

struct Stub;
impl FromStr for Stub {
    type Err = ParseError;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Stub)
    }
}

struct Str(String);
impl FromStr for Str {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Str(s.to_string()))
    }
}

struct List<I>(Vec<I>);
impl<I: FromStr> FromStr for List<I> {
    type Err = I::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ips = Vec::new();

        for s in s.split(',') {
            ips.push(s.parse()?);
        }

        Ok(List(ips))
    }
}

enum WgPropKind {
    PublicKey,
    PresharedKey,
    Endpoint,
    AdvertiseRoutes,
    AllowedIPs,
    PersistentKeepalive,
    Unknown,
    PrivateKey,
    Address,
    ListenPort,
    PostUp,
    PostDown,
    PreUp,
    PreDown,
    FWMark,
    Table,
    MTU,
    DNS,
}

impl FromStr for WgPropKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PublicKey" => WgPropKind::PublicKey,
            "PresharedKey" => WgPropKind::PresharedKey,
            "Endpoint" => WgPropKind::Endpoint,
            "AllowedIPs" => WgPropKind::AllowedIPs,
            "PersistentKeepalive" => WgPropKind::PersistentKeepalive,
            "PrivateKey" => WgPropKind::PrivateKey,
            "ListenPort" => WgPropKind::ListenPort,
            "PostUp" => WgPropKind::PostUp,
            "PostDown" => WgPropKind::PostDown,
            "PreUp" => WgPropKind::PreUp,
            "PreDown" => WgPropKind::PreDown,
            "Fwmark" => WgPropKind::FWMark,
            "DNS" => WgPropKind::DNS,
            "MTU" => WgPropKind::MTU,
            "Address" => WgPropKind::Address,
            "Table" => WgPropKind::Table,
            _ => WgPropKind::Unknown,
        })
    }
}

fn until<M: FromStr>(p: char, input: &mut &str) -> Result<M, M::Err> {
    let eol = input.chars().position(|x| x == p);
    let cnt = eol.unwrap_or(input.len());
    let res = input[0..cnt].trim().parse()?;
    *input = &input[cnt..];

    if !input.is_empty() {
        *input = &input[1..];
    }

    Ok(res)
}

impl WgConfig {
    pub fn parse_config(input: &mut &str) -> Result<Self, ParseError> {
        let mut interface = None;
        let mut peers = Vec::new();

        while !input.is_empty() {
            if input.starts_with("[Interface]") {
                interface = Some(WgConfigInterface::parse(input)?);
            } else if input.starts_with("[Peer]") {
                peers.push(WgConfigPeer::parse(input)?);
            } else {
                let _ = until::<Stub>('\n', input);
            }
        }

        Ok(WgConfig {
            interface: interface.ok_or(ParseError::NoIntrerfaceSection)?,
            peers,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::wg::{
        Endpoint, Key,
        config::{Cidr, WgConfigInterface, WgConfigPeer, WgPeerInfo},
        instance::WgInterfaceInfo,
    };

    use super::WgConfig;

    #[test]
    fn test_parse_config() {
        let priv_key = Key::random();
        let srv_key = Key::random();
        let tag_key = Key::random();
        let phone_key = Key::random();
        let kvm_key = Key::random();

        let cfg = format!(
            "[Interface]
PrivateKey = {}
Address = 100.64.0.2/24
ListenPort = 51822
PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o tun0 -j MASQUERADE
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o tun0 -j MASQUERADE

[Peer] # Server
PublicKey = {}
Endpoint = example.com:51821
AllowedIPs = 100.64.0.1, 192.168.0.0/24, 192.168.1.1
PersistentKeepalive = 25

[Peer] # Laptop
PublicKey = {}
AllowedIPs = 100.64.0.3
PersistentKeepalive = 25

[Peer] # Phone
PublicKey = {}
AllowedIPs = 100.64.0.4
PersistentKeepalive = 25

[Peer] # NanoKVM
PublicKey = {}
AllowedIPs = 100.64.0.100
PersistentKeepalive = 25",
            priv_key, srv_key, tag_key, phone_key, kvm_key,
        );

        let mut input = cfg.as_str();

        let cfg = WgConfig::parse_config(&mut input).unwrap();

        assert_eq!(
                    cfg,
                    WgConfig {
                        interface: WgConfigInterface {
                            private_key: priv_key,
                            address: Cidr {
                                ip: Ipv4Addr::new(100, 64, 0, 2).into(),
                                mask: 24
                            },
                            listen_port: Some(51822),
                            mtu: None,
                            dns: None,
                            table: None,
                            fwmark: None,
                            pre_up: None,
                            pre_down: None,
                            post_up: Some("iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o tun0 -j MASQUERADE".to_string()),
                            post_down: Some("iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o tun0 -j MASQUERADE".to_string()),
                            save_config: None,
                            advertise_routes: None
                        },
                        peers: vec![
                            WgConfigPeer {
                                public_key: srv_key,
                                preshared_key: None,
                                endpoint: Some(Endpoint::Domain("example.com:51821".to_string())),
                                allowed_ips: Some(vec![
                                    Cidr {
                                        ip: Ipv4Addr::new(100, 64, 0, 1).into(),
                                        mask: 32
                                    },
                                    Cidr {
                                        ip: Ipv4Addr::new(192, 168, 0, 0).into(),
                                        mask: 24
                                    },
                                    Cidr {
                                        ip: Ipv4Addr::new(192, 168, 1, 1).into(),
                                        mask: 32
                                    },
                                ]),
                                persistent_keepalive: Some(25),
                            },
                            WgConfigPeer {
                                public_key: tag_key,
                                preshared_key: None,
                                endpoint: None,
                                allowed_ips: Some(vec![
                                    Cidr {
                                        ip: Ipv4Addr::new(100, 64, 0, 3).into(),
                                        mask: 32
                                    },
                                ]),
                                persistent_keepalive: Some(25),
                            },
                            WgConfigPeer {
                                public_key: phone_key,
                                preshared_key: None,
                                endpoint: None,
                                allowed_ips: Some(vec![
                                    Cidr {
                                        ip: Ipv4Addr::new(100, 64, 0, 4).into(),
                                        mask: 32
                                    },
                                ]),
                                persistent_keepalive: Some(25),
                            },
                            WgConfigPeer {
                                public_key: kvm_key,
                                preshared_key: None,
                                endpoint: None,
                                allowed_ips: Some(vec![
                                    Cidr {
                                        ip: Ipv4Addr::new(100, 64, 0, 100).into(),
                                        mask: 32
                                    },
                                ]),
                                persistent_keepalive: Some(25),
                            },
                        ]
                    }
                )
    }
}
