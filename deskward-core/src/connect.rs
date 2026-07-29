//! Client connect over Tailscale IP: handshake + password auth inside Noise.

use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::auth::password::{verify_password, StoredPasswordHash};
use crate::crypto::Identity;
use crate::io_framed::{read_message, write_message};
use crate::protocol::Message;
use crate::secure::SecureTransport;
use crate::session::{Session, SessionState};
use crate::session_channel::{read_secure, secure_handshake_initiator, secure_handshake_responder, write_secure};
use crate::tailscale::TailscaleStatus;
use crate::{Error, Result};

pub const SESSION_PORT: u16 = 29118;

/// Resolve peer by MagicDNS name (case-insensitive) or IPv4 string.
pub fn resolve_peer<'a>(status: &'a TailscaleStatus, name_or_ip: &str) -> Result<&'a crate::tailscale::TailscalePeer> {
    let needle = name_or_ip.to_lowercase();
    status
        .peers
        .iter()
        .find(|p| {
            p.name.to_lowercase() == needle
                || p.ipv4 == needle
                || p.name.to_lowercase().starts_with(&format!("{needle}."))
        })
        .ok_or_else(|| Error::PeerNotFound(name_or_ip.to_string()))
}

pub async fn connect_to_peer(
    peer_ipv4: &str,
    controller_id: &str,
    password: &str,
    identity: Identity,
) -> Result<crate::session_runtime::ClientSession> {
    let addr: SocketAddr = format!("{peer_ipv4}:{SESSION_PORT}")
        .parse()
        .map_err(|e| Error::Protocol(format!("bad peer address: {e}")))?;
    let mut stream = TcpStream::connect(addr).await?;
    run_client_handshake(&mut stream, controller_id, identity).await?;
    let mut secure = secure_handshake_initiator(&mut stream).await?;
    authenticate_secure(&mut stream, &mut secure, password, peer_ipv4).await?;
    crate::session_runtime::start_client_session(stream, secure).await
}

pub async fn run_client_handshake(
    stream: &mut TcpStream,
    peer_id: &str,
    identity: Identity,
) -> Result<()> {
    let mut session = Session::new(peer_id, identity);
    let hello = session.begin_handshake();
    write_message(stream, &hello).await?;
    let ack = read_message(stream).await?;
    session.on_ack(&ack)?;
    assert_eq!(session.state, SessionState::Established);
    Ok(())
}

pub async fn authenticate_secure(
    stream: &mut TcpStream,
    secure: &mut SecureTransport,
    password: &str,
    peer_id: &str,
) -> Result<()> {
    write_secure(
        stream,
        secure,
        &Message::SessionAuth {
            password: password.to_string(),
        },
    )
    .await?;
    match read_secure(stream, secure).await? {
        Message::SessionAuthResult { ok: true, .. } => {
            let _ = crate::audit::log_session(peer_id, "session_connect", "client");
            Ok(())
        }
        Message::SessionAuthResult { ok: false, reason: _ } => Err(Error::AuthFailed),
        other => Err(Error::Protocol(format!(
            "expected SessionAuthResult, got {other:?}"
        ))),
    }
}

pub async fn accept_handshake(stream: &mut TcpStream, host_id: &str, identity: Identity) -> Result<()> {
    let hello = read_message(stream).await?;
    let mut session = Session::new(host_id, identity);
    let ack = session.on_hello(&hello)?;
    write_message(stream, &ack).await?;
    assert_eq!(session.state, SessionState::Established);
    Ok(())
}

pub async fn verify_client_password_secure(
    stream: &mut TcpStream,
    secure: &mut SecureTransport,
    stored: Option<&StoredPasswordHash>,
    host_id: &str,
) -> Result<bool> {
    let msg = read_secure(stream, secure).await?;
    let Message::SessionAuth { password } = msg else {
        return Err(Error::Protocol("expected SessionAuth".into()));
    };
    let ok = match stored {
        Some(hash) => verify_password(&password, hash).unwrap_or(false),
        None => false,
    };
    let reason = if ok {
        None
    } else {
        Some("wrong password".into())
    };
    write_secure(
        stream,
        secure,
        &Message::SessionAuthResult { ok, reason },
    )
    .await?;
    if ok {
        let _ = crate::audit::log_session(host_id, "session_connect", "host");
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn handshake_noise_auth_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let hash = crate::auth::password::hash_password("correct horse battery staple").unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let host_id = Identity::generate();
            accept_handshake(&mut stream, "host", host_id).await.unwrap();
            let mut secure = secure_handshake_responder(&mut stream).await.unwrap();
            assert!(verify_client_password_secure(&mut stream, &mut secure, Some(&hash), "host")
                .await
                .unwrap());
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let ctrl_id = Identity::generate();
        run_client_handshake(&mut stream, "ctrl", ctrl_id)
            .await
            .unwrap();
        let mut secure = secure_handshake_initiator(&mut stream).await.unwrap();
        authenticate_secure(
            &mut stream,
            &mut secure,
            "correct horse battery staple",
            "127.0.0.1",
        )
        .await
        .unwrap();
        server.await.unwrap();
    }
}
