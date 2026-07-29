//! Shared admin API token check (deskward-id admin, deskward-console).

/// Max JSON frame body (16 MiB — large JPEG/H264 keyframes).
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// `DESKWARD_ADMIN_TOKEN` when set; empty env = no token required (loopback-only deploy).
pub fn admin_token() -> Option<String> {
    std::env::var("DESKWARD_ADMIN_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
}

/// Bearer token match. When token unset, allow (caller must bind loopback).
pub fn bearer_authorized(auth_header: Option<&str>) -> bool {
    match admin_token() {
        None => true,
        Some(expected) => auth_header
            .and_then(|h| h.strip_prefix("Bearer "))
            .is_some_and(|got| got == expected),
    }
}
