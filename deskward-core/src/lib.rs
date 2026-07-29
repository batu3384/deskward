//! Deskward core: protocol v0, crypto, session and platform traits.

pub mod crypto;
pub mod error;
pub mod features;
pub mod input;
pub mod media;
pub mod protocol;
pub mod session;

pub use error::{Error, Result};
