//! Deskward ID server — register, heartbeat, punch coordination.

mod relay_registry;

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use deskward_core::protocol::{decode_frame, encode_frame, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

const DEFAULT_TCP: &str = "0.0.0.0:29115";
const PEER_TTL: Duration = Duration::from_secs(90);

#[derive(Clone)]
struct PeerRecord {
    endpoint: String,
    last_seen: Instant,
}

#[derive(Clone)]
struct Registry {
    peers: Arc<DashMap<String, PeerRecord>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            peers: Arc::new(DashMap::new()),
        }
    }

    fn register(&self, peer_id: String, endpoint: String) {
        self.peers.insert(
            peer_id.clone(),
            PeerRecord {
                endpoint,
                last_seen: Instant::now(),
            },
        );
        info!(peer_id, "registered");
    }

    fn heartbeat(&self, peer_id: &str) -> bool {
        if let Some(mut rec) = self.peers.get_mut(peer_id) {
            rec.last_seen = Instant::now();
            true
        } else {
            false
        }
    }

    fn endpoint_of(&self, peer_id: &str) -> Option<String> {
        self.peers.get(peer_id).map(|p| p.endpoint.clone())
    }

    fn prune_stale(&self) {
        self.peers
            .retain(|_, rec| rec.last_seen.elapsed() < PEER_TTL);
    }
}

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
        Message::PunchRequest { from, to } => {
            let endpoint = registry.endpoint_of(&to)?;
            let relay = relay_registry::pick_relay();
            let resp = Message::PunchResponse {
                from: to,
                to: from,
                endpoint,
            };
            if let Some(r) = relay {
                tracing::debug!(relay_host = %r.host, relay_port = r.port, "relay hint available");
            }
            Some(resp)
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
    let registry = Registry::new();
    let listener = TcpListener::bind(&bind).await?;
    info!(%bind, "deskward-id listening");

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
