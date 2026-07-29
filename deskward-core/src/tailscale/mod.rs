pub mod mock;
pub mod system;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailscalePeer {
    pub name: String,
    pub ipv4: String,
    pub online: bool,
    pub os: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TailscaleStatus {
    pub installed: bool,
    pub running: bool,
    pub self_name: Option<String>,
    pub self_ipv4: Option<String>,
    pub peers: Vec<TailscalePeer>,
}

#[async_trait]
pub trait TailscaleClient: Send + Sync {
    async fn status(&self) -> Result<TailscaleStatus>;
}
