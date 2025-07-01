use std::{fs, pin::pin};

use clap::Parser;
use discover::Discover;
use error::Error;
use futures::StreamExt;
use signaling::{
    PeerUpdate, Signaling,
    irc::{IrcConfig, IrcSignaling, PeerEvent},
};
use wg::{Key, WireguardApi, cmd::WgCmdBackend, config::WgConfig};

mod discover;
pub(crate) mod error;
mod signaling;
mod wg;

#[derive(Debug, clap::Parser)]
pub struct Args {
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    unsafe { std::env::set_var("RUST_LOG", "info") };
    env_logger::init();

    let args = Args::parse();
    let config = load_wg_config(&args.iface)?;
    let iface = args.iface;

    let mut wg = WgCmdBackend::new();
    let key = wg.get_pub_key(&iface)?;

    let cfg = IrcConfig {
        server: "irc.libera.chat".to_string(),
        port: Some(6667),
        tls: false,
        channel: "#wg-disco-aeeab".to_string(),
    };

    let mut signaling =
        IrcSignaling::connect(cfg, key, config.peers.iter().map(|x| &x.public_key)).await?;

    let discover = discover::stun::StunDiscover::default();
    let (endpoint, local_port) = discover.discover().await?;

    if config.interface.listen_port.is_none() {
        wg.set_listen_port(&iface, local_port)?;
    };

    // announcing self peer
    signaling
        .announce(
            PeerUpdate {
                key,
                endpoint,
                advertise_routes: vec![],
            },
            None,
        )
        .await?;

    let mut stream = pin!(signaling.subscribe().await?);

    while let Some(res) = stream.next().await {
        match res {
            Ok(PeerEvent::Request(nick, peer)) => {
                // update peers endpoint
                log::info!(
                    "requested update from {} peer {} {}",
                    nick,
                    peer.key,
                    peer.endpoint
                );
                wg.set_peer_endpoint(&iface, peer.key, peer.endpoint.into())?;

                signaling
                    .announce(
                        PeerUpdate {
                            key,
                            endpoint,
                            advertise_routes: vec![],
                        },
                        Some(&nick),
                    )
                    .await?;
            }

            Ok(PeerEvent::Response(peer)) => {
                // update peers endpoint
                log::info!("responded update peer {} {}", peer.key, peer.endpoint);

                wg.set_peer_endpoint(&iface, peer.key, peer.endpoint.into())?;
            }

            Err(err) => log::error!("error: {err}"),
        }
    }

    println!("exit");

    Ok(())
}

fn load_wg_config(iface: &str) -> Result<WgConfig, Error> {
    let data = fs::read_to_string(format!("/etc/wireguard/{iface}.conf"))?;
    let mut reader = data.as_str();

    Ok(WgConfig::parse_config(&mut reader)?)
}
