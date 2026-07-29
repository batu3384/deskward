//! Deskward core: protocol v0, crypto, session and platform traits.

pub mod acl;
pub mod admin_auth;
pub mod auth;
pub mod audit;
pub mod checklist;
pub mod connect;
pub mod crypto;
pub mod error;
pub mod features;
pub mod host_gate;
pub mod input;
pub mod io_framed;
pub mod media;
pub mod perf;
pub mod protocol;
pub mod role;
pub mod secure;
pub mod session;
pub mod session_channel;
pub mod session_handlers;
pub mod session_runtime;
pub mod tailscale;

pub use error::{Error, Result};
