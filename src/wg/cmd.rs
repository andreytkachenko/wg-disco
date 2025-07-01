use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use crate::error::Error;

use super::{Endpoint, Key, WireguardApi, config::ParseError};

pub struct WgCmdBackend;
impl WgCmdBackend {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl WireguardApi for WgCmdBackend {
    type Error = Error;

    fn get_pub_key(&self, iface: &str) -> Result<Key, Self::Error> {
        let out = std::process::Command::new("wg")
            .arg("show")
            .arg(iface)
            .arg("public-key")
            .output()?;

        if !out.status.success() {
            return Err(Error::WgCommandFail(out.status.code()));
        }

        let key_str = unsafe { String::from_utf8_unchecked(out.stdout) };

        Ok(Key::from_str(key_str.trim()).map_err(ParseError::from)?)
    }

    fn get_listen_port(&self, iface: &str) -> Result<u16, Self::Error> {
        let out = std::process::Command::new("wg")
            .arg("show")
            .arg(iface)
            .arg("listen-port")
            .output()?;

        if !out.status.success() {
            return Err(Error::WgCommandFail(out.status.code()));
        }

        let port_str = unsafe { String::from_utf8_unchecked(out.stdout) };

        Ok(u16::from_str(port_str.trim()).map_err(ParseError::from)?)
    }

    fn get_endpoints(
        &self,
        iface: &str,
    ) -> Result<std::collections::HashMap<Key, Option<SocketAddr>>, Self::Error> {
        let out = std::process::Command::new("wg")
            .arg("show")
            .arg(iface)
            .arg("endpoints")
            .output()?;
        if !out.status.success() {
            return Err(Error::WgCommandFail(out.status.code()));
        }
        let mut map = HashMap::new();
        let table = unsafe { String::from_utf8_unchecked(out.stdout) };
        for line in table.lines() {
            let (key_str, addr_str) = line.split_once(char::is_whitespace).unwrap();
            let key = Key::from_str(key_str.trim()).map_err(ParseError::from)?;
            let addr = SocketAddr::from_str(addr_str.trim()).ok();
            map.insert(key, addr);
        }
        Ok(map)
    }

    fn set_listen_port(&mut self, iface: &str, port: u16) -> Result<(), Self::Error> {
        let out = std::process::Command::new("wg")
            .arg("set")
            .arg(iface)
            .arg("listen-port")
            .arg(port.to_string())
            .output()?;

        if !out.status.success() {
            return Err(Error::WgCommandFail(out.status.code()));
        }

        Ok(())
    }

    fn set_peer_endpoint(
        &mut self,
        iface: &str,
        key: Key,
        endpoint: Endpoint,
    ) -> Result<(), Self::Error> {
        let out = std::process::Command::new("wg")
            .arg("set")
            .arg(iface)
            .arg("peer")
            .arg(key.to_string())
            .arg("endpoint")
            .arg(endpoint.to_string())
            .output()?;

        if !out.status.success() {
            return Err(Error::WgCommandFail(out.status.code()));
        }

        Ok(())
    }
}
