mod agent;
mod capture;
mod input;

use tracing::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_host_macos=info".parse().unwrap()),
        )
        .init();

    let peer_id = std::env::var("DESKWARD_PEER_ID").unwrap_or_else(|_| "mac-host".into());
    let id_addr = std::env::var("DESKWARD_ID_ADDR").unwrap_or_else(|_| "127.0.0.1:29115".into());
    let endpoint = std::env::var("DESKWARD_ENDPOINT").unwrap_or_else(|_| "127.0.0.1:0".into());

    let _capture = capture::MacScreenCapture::new();
    let _input = input::MacInputInjector::new();
    info!("deskward-host-macos starting (capture/input stubs active)");

    agent::run_agent(&peer_id, &id_addr, &endpoint).await
}
