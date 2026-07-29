//! System Tailscale LocalAPI client (macOS TCP / Unix socket).

use async_trait::async_trait;
use tailscale_localapi::types::BackendState;

use super::{TailscaleClient, TailscalePeer, TailscaleStatus};
use crate::{Error, Result};

pub struct SystemTailscale;

impl SystemTailscale {
    pub fn new() -> Self {
        Self
    }

    async fn fetch_async() -> Result<TailscaleStatus> {
        let status = platform::local_status()
            .await
            .map_err(|e| Error::Tailscale(e.to_string()))?;

        let self_name = if status.self_status.dnsname.is_empty() {
            status.self_status.hostname.clone()
        } else {
            status.self_status.dnsname.clone()
        };

        let self_ipv4 = status
            .tailscale_ips
            .iter()
            .chain(status.self_status.tailscale_ips.iter())
            .find(|ip| ip.is_ipv4())
            .map(|ip| ip.to_string());

        let running = matches!(status.backend_state, BackendState::Running);

        let peers: Vec<TailscalePeer> = status
            .peer
            .values()
            .map(|p| {
                let ipv4 = p
                    .tailscale_ips
                    .iter()
                    .find(|ip| ip.is_ipv4())
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let name = if p.dnsname.is_empty() {
                    p.hostname.clone()
                } else {
                    p.dnsname.clone()
                };
                TailscalePeer {
                    name,
                    ipv4,
                    online: p.online,
                    os: p.os.clone(),
                }
            })
            .collect();

        Ok(TailscaleStatus {
            installed: true,
            running,
            self_name: Some(self_name),
            self_ipv4,
            peers,
        })
    }

    fn fetch_sync() -> Result<TailscaleStatus> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Tailscale(e.to_string()))?;
        rt.block_on(Self::fetch_async())
    }
}

impl Default for SystemTailscale {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TailscaleClient for SystemTailscale {
    async fn status(&self) -> Result<TailscaleStatus> {
        Self::fetch_async().await
    }
}

pub fn fetch_status() -> Result<TailscaleStatus> {
    SystemTailscale::fetch_sync()
}

mod platform {
    #[cfg(target_os = "macos")]
    pub async fn local_status(
    ) -> std::result::Result<tailscale_localapi::types::Status, tailscale_localapi::Error> {
        use std::fs;
        use std::path::PathBuf;

        let dir = PathBuf::from("/Library/Tailscale");
        let port: u16 = fs::read_link(dir.join("ipnport"))
            .map_err(|e| tailscale_localapi::Error::IoError(e))?
            .to_string_lossy()
            .parse()
            .map_err(|e| {
                tailscale_localapi::Error::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                ))
            })?;
        let password_path = dir.join(format!("sameuserproof-{port}"));
        let password = fs::read_to_string(&password_path)
            .map_err(|e| tailscale_localapi::Error::IoError(e))?
            .trim_end()
            .to_string();
        let api = tailscale_localapi::LocalApi::new_with_port_and_password(port, password);
        api.status().await
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub async fn local_status(
    ) -> std::result::Result<tailscale_localapi::types::Status, tailscale_localapi::Error> {
        let api = tailscale_localapi::LocalApi::new_with_socket_path(
            "/var/run/tailscale/tailscaled.sock",
        );
        api.status().await
    }

    #[cfg(not(unix))]
    pub async fn local_status(
    ) -> std::result::Result<tailscale_localapi::types::Status, tailscale_localapi::Error> {
        Err(tailscale_localapi::Error::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Tailscale LocalAPI not supported on this platform",
        )))
    }
}
