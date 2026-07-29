//! HTTP admin API for console / operators.

use std::net::SocketAddr;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use deskward_core::admin_auth;
use tracing::{info, warn};

use crate::registry::Registry;

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

pub async fn run_admin(registry: Registry, bind: String) -> std::io::Result<()> {
    if admin_auth::admin_token().is_none() && !bind.starts_with("127.0.0.1") {
        warn!("DESKWARD_ADMIN_TOKEN unset while admin binds non-loopback — set token in production");
    }

    let peers = Router::new().route(
        "/api/v1/peers",
        get({
            let reg = registry.clone();
            move || async move {
                let peers = reg.list_peers();
                Json(peers)
            }
        }),
    );

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(peers.layer(middleware::from_fn(require_admin_token)));

    let addr: SocketAddr = bind.parse().map_err(std::io::Error::other)?;
    info!(%addr, "deskward-id admin HTTP listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}
