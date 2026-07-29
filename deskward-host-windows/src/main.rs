mod agent;
mod capture;
mod clipboard;
mod gate;
mod input;
mod session;

use tracing::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_host_windows=info".parse().unwrap()),
        )
        .init();

    if !gate::setup_gate_open()? {
        eprintln!(
            "deskward-host-windows: setup gate closed — complete checklist and arm host, \
             or set DESKWARD_FORCE_LISTEN=1 for dev"
        );
        std::process::exit(1);
    }

    let peer_id = std::env::var("DESKWARD_PEER_ID").unwrap_or_else(|_| "win-host".into());
    let id_addr = std::env::var("DESKWARD_ID_ADDR").unwrap_or_else(|_| "127.0.0.1:29115".into());
    let endpoint = std::env::var("DESKWARD_ENDPOINT").unwrap_or_else(|_| "127.0.0.1:0".into());

    info!("deskward-host-windows starting");

    let bind_ip = std::env::var("DESKWARD_TAILSCALE_IP").unwrap_or_else(|_| {
        deskward_core::tailscale::system::fetch_status()
            .ok()
            .and_then(|s| s.self_ipv4)
            .unwrap_or_else(|| "127.0.0.1".into())
    });

    let password_hash = gate::password_hash_from_setup();
    let session_peer = peer_id.clone();
    tokio::spawn(async move {
        if let Err(e) = session::run_session_listener(&bind_ip, &session_peer, password_hash).await
        {
            tracing::error!(?e, "session listener exited");
        }
    });

    agent::run_agent(&peer_id, &id_addr, &endpoint).await
}
