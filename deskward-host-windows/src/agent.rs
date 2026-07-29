//! Register host with deskward-id and run heartbeat loop.

use deskward_core::protocol::{encode_frame, Message};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::info;

pub async fn run_agent(peer_id: &str, id_addr: &str, endpoint: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(id_addr).await?;
    let reg = Message::Register {
        peer_id: peer_id.to_string(),
        endpoint: endpoint.to_string(),
    };
    stream
        .write_all(&encode_frame(&reg).map_err(std::io::Error::other)?)
        .await?;
    info!(peer_id, id_addr, "registered with deskward-id");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        let hb = Message::Heartbeat {
            peer_id: peer_id.to_string(),
        };
        stream
            .write_all(&encode_frame(&hb).map_err(std::io::Error::other)?)
            .await?;
    }
}
