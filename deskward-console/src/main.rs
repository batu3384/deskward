mod audit;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{routing::get, Json, Router};
use serde::Serialize;
use tracing::info;

#[derive(Clone, Serialize)]
struct Device {
    peer_id: String,
    endpoint: String,
    online: bool,
}

#[derive(Clone, Default)]
struct Registry {
    devices: Arc<Mutex<Vec<Device>>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_console=info".parse().unwrap()),
        )
        .init();

    let registry = Registry::default();
    {
        let mut d = registry.devices.lock().unwrap();
        *d = vec![
            Device {
                peer_id: "mac-host".into(),
                endpoint: "192.168.1.20:0".into(),
                online: true,
            },
            Device {
                peer_id: "win-desk".into(),
                endpoint: "192.168.1.30:0".into(),
                online: false,
            },
        ];
    }

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/api/v1/devices",
            get({
                let reg = registry.clone();
                move || async move {
                    let list = reg.devices.lock().unwrap().clone();
                    Json(list)
                }
            }),
        )
        .route(
            "/api/v1/groups",
            get(|| async { Json(serde_json::json!({ "groups": [] })) }),
        );

    let bind = std::env::var("DESKWARD_CONSOLE_ADDR").unwrap_or_else(|_| "0.0.0.0:29200".into());
    let addr: SocketAddr = bind.parse().map_err(std::io::Error::other)?;
    info!(%addr, "deskward-console listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}
