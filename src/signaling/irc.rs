use std::{collections::HashMap, hash::Hash};

use base64::{Engine, prelude::BASE64_URL_SAFE};
use bincode::config::{BigEndian, Configuration};
use futures::TryStreamExt;
use hashes::sha2::sha256;
use irc::{
    client::{Client, data::Config},
    proto::{Command, Prefix},
};

use crate::{error::Error, wg::Key};

use super::{PeerUpdate, Signaling};

const BINCODE_CONFIG: Configuration<BigEndian> = bincode::config::standard().with_big_endian();
const NICKNAME_LENGTH: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerEvent {
    Request(String, PeerUpdate),
    Response(PeerUpdate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrcConfig {
    pub server: String,
    pub port: Option<u16>,
    pub tls: bool,
    pub channel: String,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
struct Nickname([u8; NICKNAME_LENGTH]);

impl From<[u8; 44]> for Nickname {
    fn from(username: [u8; 44]) -> Nickname {
        let mut buf = [0; NICKNAME_LENGTH];
        buf.copy_from_slice(&username[0..NICKNAME_LENGTH]);
        Nickname(buf)
    }
}

impl std::fmt::Debug for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Nickname").field(&self.0).finish()
    }
}

impl std::fmt::Display for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", str::from_utf8(&self.0).unwrap_or("<err>"))
    }
}

pub struct IrcSignaling {
    channel: String,
    client: Client,
    registry: HashMap<Nickname, Key>,
    nickname: String,
}

impl IrcSignaling {
    pub async fn connect(
        config: IrcConfig,
        pub_key: Key,
        peers: impl IntoIterator<Item = &Key>,
    ) -> Result<Self, irc::error::Error> {
        let username = str::from_utf8(&Self::username(&pub_key))
            .unwrap()
            .to_string()
            .replace(['-', '_'], "");

        let nickname = format!("{}", &username[..NICKNAME_LENGTH]);
        let mut registry = HashMap::new();

        for key in peers {
            registry.insert(Self::username(key).into(), *key);
        }

        let client = Client::from_config(Config {
            username: Some(username),
            nickname: Some(nickname.clone()),
            server: Some(config.server),
            port: config.port,
            use_tls: Some(config.tls),

            ..Default::default()
        })
        .await?;

        client.identify()?;
        client.send_join(&config.channel)?;

        Ok(Self {
            client,
            channel: config.channel,
            nickname,
            registry,
        })
    }

    #[inline]
    fn username(key: &Key) -> [u8; 44] {
        let mut username = [0u8; 44];
        BASE64_URL_SAFE
            .encode_slice(sha256::hash(key.as_ref()).into_bytes(), &mut username)
            .unwrap();
        username
    }

    #[inline]
    fn encode_msg(peer: &PeerUpdate) -> Result<String, Error> {
        Ok(BASE64_URL_SAFE.encode(bincode::encode_to_vec(
            &peer,
            bincode::config::standard().with_big_endian(),
        )?))
    }

    #[inline]
    fn decode_msg(msg: &str) -> Result<PeerUpdate, Error> {
        let msg = BASE64_URL_SAFE.decode(msg)?;

        Ok(bincode::decode_from_slice(&msg, BINCODE_CONFIG)?.0)
    }
}

impl Signaling for IrcSignaling {
    type Error = Error;

    async fn subscribe(
        &mut self,
    ) -> Result<impl futures::Stream<Item = Result<PeerEvent, Self::Error>> + use<>, Self::Error>
    {
        let channel = self.channel.clone();
        let nickname = self.nickname.clone();

        Ok(self
            .client
            .stream()?
            .map_err(Error::IrcError)
            .try_filter_map(move |x| {
                let channel = channel.clone();
                let nickname = nickname.clone();

                async move {
                    println!("msg {:?} {:?}", x.prefix, x.command);

                    Ok(match x.command {
                        Command::PRIVMSG(target, msg) => {
                            if let Some(Prefix::Nickname(nm, _, _)) = x.prefix {
                                let msg = Self::decode_msg(&msg).ok();

                                if target == channel {
                                    msg.map(|upd| PeerEvent::Request(nm, upd))
                                } else {
                                    msg.map(PeerEvent::Response)
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                }
            }))
    }

    async fn announce(&mut self, peer: PeerUpdate, nick: Option<&str>) -> Result<(), Self::Error> {
        let target = nick.unwrap_or(&self.channel);

        log::info!(
            "announcing peer for {} {} {}",
            target,
            peer.key,
            peer.endpoint
        );

        let msg = Self::encode_msg(&peer)?;
        self.client.send_privmsg(target, msg)?;
        Ok(())
    }
}
