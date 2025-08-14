pub mod contracts;
pub mod enums;
pub mod error;
mod ext;
pub mod helpers;
pub mod http;
pub mod server;
pub mod setup;

pub use setup::state::FoxtiveAxumState;
use std::sync::OnceLock;

pub static FOXTIVE_AXUM: OnceLock<FoxtiveAxumState> = OnceLock::new();

pub use crate::ext::app_state::FoxtiveAxumExt;
