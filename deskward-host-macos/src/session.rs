//! Session listener on Tailscale IP only.

use std::sync::Arc;

use deskward_core::auth::password::StoredPasswordHash;
use deskward_core::connect::{
    accept_handshake, verify_client_password_secure, SESSION_PORT,
};
use deskward_core::session_channel::secure_handshake_responder;
use deskward_core::crypto::Identity;
use deskward_core::features::clipboard::ClipboardSync;
use deskward_core::features::file_receive::FileReceiver;
use deskward_core::features::recording::{NoopRecording, RecordingSink};
use deskward_core::session_handlers::{clipboard_enabled, file_receive_enabled, recording_enabled, HostHandlers};
use deskward_core::session_runtime::run_host_session;
use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::capture::MacScreenCapture;
use crate::clipboard::HostClipboard;
use crate::input::MacInputInjector;

pub async fn run_session_listener(
    bind_ip: &str,
    host_id: &str,
    password_hash: Option<StoredPasswordHash>,
) -> std::io::Result<()> {
    let addr = format!("{bind_ip}:{SESSION_PORT}");
    let listener = TcpListener::bind(&addr).await?;
    info!(%addr, "session listener bound (tailnet only)");
    let host_id = host_id.to_string();
    let hash = Arc::new(password_hash);

    loop {
        let (stream, peer) = listener.accept().await?;
        let host_id = host_id.clone();
        let hash = hash.clone();
        tokio::spawn(async move {
            let mut stream = stream;
            let identity = Identity::generate();
            if let Err(e) = accept_handshake(&mut stream, &host_id, identity).await {
                warn!(?e, %peer, "handshake failed");
                return;
            }
            let mut secure = match secure_handshake_responder(&mut stream).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(?e, %peer, "noise handshake failed");
                    return;
                }
            };
            match verify_client_password_secure(
                &mut stream,
                &mut secure,
                hash.as_ref().as_ref(),
                &host_id,
            )
            .await
            {
                Ok(true) => info!(%peer, "session authenticated"),
                Ok(false) => {
                    warn!(%peer, "auth rejected");
                    return;
                }
                Err(e) => {
                    warn!(?e, %peer, "auth error");
                    return;
                }
            }

            let capture = match MacScreenCapture::new() {
                Ok(c) => c,
                Err(e) => {
                    warn!(?e, "capture init failed");
                    return;
                }
            };
            let input = match MacInputInjector::new() {
                Ok(i) => i,
                Err(e) => {
                    warn!(?e, "input init failed");
                    return;
                }
            };

            let mut clipboard = HostClipboard::new().ok();
            let mut files =
                FileReceiver::new(std::env::temp_dir().join("deskward-incoming")).ok();
            let mut recording = NoopRecording;
            let handlers = HostHandlers {
                clipboard: if clipboard_enabled() {
                    clipboard
                        .as_mut()
                        .map(|c| c as &mut dyn ClipboardSync)
                } else {
                    None
                },
                files: if file_receive_enabled() {
                    files.as_mut()
                } else {
                    None
                },
                recording: if recording_enabled() {
                    Some(&mut recording as &mut dyn RecordingSink)
                } else {
                    None
                },
            };

            if let Err(e) = run_host_session(stream, secure, capture, input, handlers).await {
                warn!(?e, %peer, "host session ended");
            }
        });
    }
}
