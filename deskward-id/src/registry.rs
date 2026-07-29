//! In-memory peer registry for deskward-id.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use serde::Serialize;
use tracing::info;

pub const PEER_TTL: Duration = Duration::from_secs(90);

#[derive(Clone)]
struct PeerRecord {
    endpoint: String,
    last_seen: Instant,
}

#[derive(Clone, Serialize)]
pub struct PeerSnapshot {
    pub peer_id: String,
    pub endpoint: String,
    pub online: bool,
}

#[derive(Clone)]
pub struct Registry {
    peers: Arc<DashMap<String, PeerRecord>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, peer_id: String, endpoint: String) {
        self.peers.insert(
            peer_id.clone(),
            PeerRecord {
                endpoint,
                last_seen: Instant::now(),
            },
        );
        info!(peer_id, "registered");
    }

    pub fn heartbeat(&self, peer_id: &str) -> bool {
        if let Some(mut rec) = self.peers.get_mut(peer_id) {
            rec.last_seen = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn endpoint_of(&self, peer_id: &str) -> Option<String> {
        self.peers.get(peer_id).map(|p| p.endpoint.clone())
    }

    pub fn list_peers(&self) -> Vec<PeerSnapshot> {
        self.peers
            .iter()
            .map(|entry| {
                let online = entry.value().last_seen.elapsed() < PEER_TTL;
                PeerSnapshot {
                    peer_id: entry.key().clone(),
                    endpoint: entry.value().endpoint.clone(),
                    online,
                }
            })
            .collect()
    }

    pub fn prune_stale(&self) {
        self.peers
            .retain(|_, rec| rec.last_seen.elapsed() < PEER_TTL);
    }
}
