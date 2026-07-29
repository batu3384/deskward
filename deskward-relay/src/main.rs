//! Deskward relay — pairs two TCP legs when P2P punch fails.

use std::sync::Arc;

use dashmap::DashMap;
use deskward_core::protocol::{decode_frame, encode_frame, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tracing::{info, warn};

const DEFAULT_BIND: &str = "0.0.0.0:29117";

type WaitMap = Arc<DashMap<String, oneshot::Sender<TcpStream>>>;

async fn read_allocate(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    let len = u32::from_be_bytes(header) as usize;
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await?;
    let mut frame = Vec::with_capacity(4 + len);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&body);
    let (msg, _) = decode_frame(&frame).map_err(std::io::Error::other)?;
    match msg {
        Message::RelayAllocate { session_id } => Ok(session_id),
        other => {
            warn!(?other, "expected RelayAllocate");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad allocate",
            ))
        }
    }
}

async fn pipe(mut a: TcpStream, mut b: TcpStream) -> std::io::Result<()> {
    let (mut ar, mut aw) = a.split();
    let (mut br, mut bw) = b.split();
    let ab = tokio::io::copy(&mut ar, &mut bw);
    let ba = tokio::io::copy(&mut br, &mut aw);
    tokio::try_join!(ab, ba)?;
    Ok(())
}

async fn handle_leg(mut stream: TcpStream, waiting: WaitMap) -> std::io::Result<()> {
    let session_id = read_allocate(&mut stream).await?;

    if let Some((_, sender)) = waiting.remove(&session_id) {
        info!(session_id, "relay pairing second leg");
        let _ = sender.send(stream);
        Ok(())
    } else {
        let (tx, rx) = oneshot::channel();
        waiting.insert(session_id.clone(), tx);
        let ready = encode_frame(&Message::RelayReady {
            session_id: session_id.clone(),
            relay_port: 29117,
        })
        .map_err(std::io::Error::other)?;
        stream.write_all(&ready).await?;
        info!(session_id, "relay waiting for peer");
        let peer = rx.await.map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "peer timeout")
        })?;
        pipe(stream, peer).await
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_relay=info".parse().unwrap()),
        )
        .init();

    let bind = std::env::var("DESKWARD_RELAY_ADDR").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let waiting: WaitMap = Arc::new(DashMap::new());
    let listener = TcpListener::bind(&bind).await?;
    info!(%bind, "deskward-relay listening");

    loop {
        let (stream, addr) = listener.accept().await?;
        info!(%addr, "relay connection");
        let waiting = waiting.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_leg(stream, waiting).await {
                warn!(?e, "relay leg error");
            }
        });
    }
}
