use std::sync::Mutex;

use async_trait::async_trait;

use super::{TailscaleClient, TailscalePeer, TailscaleStatus};
use crate::Result;

pub struct MockTailscale {
    inner: Mutex<TailscaleStatus>,
}

impl MockTailscale {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(TailscaleStatus::default()),
        }
    }

    pub fn set_installed(&self, installed: bool) {
        self.inner.lock().unwrap().installed = installed;
    }

    pub fn set_running(&self, running: bool) {
        self.inner.lock().unwrap().running = running;
    }

    pub fn set_self_node(&self, name: impl Into<String>, ipv4: impl Into<String>) {
        let mut s = self.inner.lock().unwrap();
        s.self_name = Some(name.into());
        s.self_ipv4 = Some(ipv4.into());
    }

    pub fn set_peers(&self, peers: Vec<TailscalePeer>) {
        self.inner.lock().unwrap().peers = peers;
    }
}

impl Default for MockTailscale {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TailscaleClient for MockTailscale {
    async fn status(&self) -> Result<TailscaleStatus> {
        Ok(self.inner.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checklist::{CheckId, CheckStatus, SetupSnapshot};
    use crate::role::DeviceRole;
    use crate::checklist::Platform;

    #[tokio::test]
    async fn peer_visible_when_online_peer_exists() {
        let mock = MockTailscale::new();
        mock.set_installed(true);
        mock.set_running(true);
        mock.set_peers(vec![TailscalePeer {
            name: "ev-mac".into(),
            ipv4: "100.64.0.1".into(),
            online: true,
            os: "macOS".into(),
        }]);

        let status = mock.status().await.unwrap();
        let mut snap = SetupSnapshot::empty(DeviceRole::Controller, Platform::MacOs);
        snap.set(
            CheckId::PeerVisible,
            if status.peers.iter().any(|p| p.online) {
                CheckStatus::Done
            } else {
                CheckStatus::ActionNeeded
            },
        );
        assert_eq!(snap.get(CheckId::PeerVisible), CheckStatus::Done);
    }
}
