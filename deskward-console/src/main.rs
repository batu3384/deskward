use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use deskward_core::acl::{self, AclGroup};
use deskward_core::admin_auth;
use deskward_core::audit;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Clone, Serialize, Deserialize)]
struct Device {
    peer_id: String,
    endpoint: String,
    online: bool,
}

#[derive(Clone)]
struct AppState {
    groups: Arc<Mutex<Vec<AclGroup>>>,
    id_admin: String,
    http: reqwest::Client,
}

async fn require_admin_token(request: Request, next: Next) -> Response {
    if admin_auth::bearer_authorized(
        request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok()),
    ) {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, "missing or invalid admin token").into_response()
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn list_devices(State(state): State<AppState>) -> Json<Vec<Device>> {
    let mut req = state
        .http
        .get(format!("{}/api/v1/peers", state.id_admin));
    if let Some(token) = admin_auth::admin_token() {
        req = req.bearer_auth(token);
    }
    match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            let peers = resp.json::<Vec<Device>>().await.unwrap_or_default();
            Json(peers)
        }
        _ => Json(vec![]),
    }
}

async fn list_groups(State(state): State<AppState>) -> Json<serde_json::Value> {
    let groups = state.groups.lock().unwrap().clone();
    Json(serde_json::json!({ "groups": groups }))
}

async fn create_group(
    State(state): State<AppState>,
    Json(body): Json<AclGroup>,
) -> Json<serde_json::Value> {
    let mut groups = state.groups.lock().unwrap();
    groups.push(body.clone());
    let _ = acl::save_groups(&groups);
    let _ = audit::log_session(&body.name, "acl_group_create", "console");
    Json(serde_json::json!({ "ok": true, "group": body }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_console=info".parse().unwrap()),
        )
        .init();

    let id_admin = std::env::var("DESKWARD_ID_ADMIN")
        .unwrap_or_else(|_| "http://127.0.0.1:29116".into());
    let state = AppState {
        groups: Arc::new(Mutex::new(acl::load_groups())),
        id_admin,
        http: reqwest::Client::new(),
    };

    let protected = Router::new()
        .route("/api/v1/devices", get(list_devices))
        .route("/api/v1/groups", get(list_groups).post(create_group))
        .layer(middleware::from_fn(require_admin_token));

    let app = Router::new()
        .route("/health", get(health))
        .merge(protected)
        .with_state(state);

    let bind = std::env::var("DESKWARD_CONSOLE_ADDR").unwrap_or_else(|_| "127.0.0.1:29200".into());
    if admin_auth::admin_token().is_none() && !bind.starts_with("127.0.0.1") {
        warn!("DESKWARD_ADMIN_TOKEN unset while console binds non-loopback — set token in production");
    }
    let addr: SocketAddr = bind.parse().map_err(std::io::Error::other)?;
    info!(%addr, "deskward-console listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}
