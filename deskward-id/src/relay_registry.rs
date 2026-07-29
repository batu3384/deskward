//! Multi-relay registry (Faz 3) — static list from env for now.

use std::env;

#[derive(Clone, Debug)]
pub struct RelayEndpoint {
    pub host: String,
    pub port: u16,
}

pub fn configured_relays() -> Vec<RelayEndpoint> {
    env::var("DESKWARD_RELAYS")
        .unwrap_or_else(|_| "127.0.0.1:29117".into())
        .split(',')
        .filter_map(|s| {
            let s = s.trim();
            let (host, port) = s.rsplit_once(':')?;
            port.parse().ok().map(|port| RelayEndpoint {
                host: host.to_string(),
                port,
            })
        })
        .collect()
}

pub fn pick_relay() -> Option<RelayEndpoint> {
    configured_relays().into_iter().next()
}
