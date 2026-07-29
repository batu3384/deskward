//! Deskward ID server — register, heartbeat, punch coordination.

mod admin;
mod registry;
mod relay_registry;

use std::time::Duration;

use deskward_core::protocol::{decode_frame, encode_frame, Message, RelayHint};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

use crate::registry::Registry;

const DEFAULT_TCP: &str = "0.0.0.0:29115";
const DEFAULT_ADMIN: &str = "127.0.0.1:29116";

async fn handle_client(mut stream: TcpStream, registry: Registry) -> std::io::Result<()> {
    let mut buf = vec![0u8; 65536];
    let mut acc = Vec::new();
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        acc.extend_from_slice(&buf[..n]);
        loop {
            match decode_frame(&acc) {
                Ok((msg, consumed)) => {
                    acc.drain(..consumed);
                    if let Some(reply) = process_message(&registry, msg).await {
                        let frame = encode_frame(&reply).map_err(std::io::Error::other)?;
                        stream.write_all(&frame).await?;
                    }
                }
                Err(deskward_core::Error::Protocol(_)) => break,
                Err(e) => {
                    warn!(?e, "protocol error");
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn process_message(registry: &Registry, msg: Message) -> Option<Message> {
    match msg {
        Message::Register { peer_id, endpoint } => {
            registry.register(peer_id, endpoint);
            None
        }
        Message::Heartbeat { peer_id } => {
            if !registry.heartbeat(&peer_id) {
                warn!(peer_id, "heartbeat for unknown peer");
            }
            None
        }
        Message::PunchRequest { from, to, region } => {
            let groups = deskward_core::acl::load_groups();
            if !deskward_core::acl::punch_allowed(&from, &to, &groups) {
                return Some(Message::PunchDenied {
                    to: from,
                    reason: "acl denied".into(),
                });
            }
            let endpoint = registry.endpoint_of(&to)?;
            let relay = relay_registry::pick_relay(region.as_deref()).map(|r| RelayHint {
                host: r.host,
                port: r.port,
            });
            Some(Message::PunchResponse {
                from: to,
                to: from,
                endpoint,
                relay,
            })
        }
        other => {
            warn!(?other, "unexpected message on id server");
            None
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_id=info".parse().unwrap()),
        )
        .init();

    let bind = std::env::var("DESKWARD_ID_ADDR").unwrap_or_else(|_| DEFAULT_TCP.to_string());
    let admin_bind =
        std::env::var("DESKWARD_ID_ADMIN_ADDR").unwrap_or_else(|_| DEFAULT_ADMIN.to_string());
    let registry = Registry::new();
    let listener = TcpListener::bind(&bind).await?;
    info!(%bind, "deskward-id listening");

    let admin_reg = registry.clone();
    tokio::spawn(async move {
        if let Err(e) = admin::run_admin(admin_reg, admin_bind).await {
            warn!(?e, "admin HTTP server stopped");
        }
    });

    let reg_prune = registry.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            reg_prune.prune_stale();
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        info!(%addr, "client connected");
        let reg = registry.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, reg).await {
                warn!(?e, "client error");
            }
        });
    }
}
