use axum::http::{HeaderName, HeaderValue, Method};
use std::fmt::{Debug, Formatter};
use crate::server::BodyConfig;

#[derive(Clone)]
pub struct FoxtiveAxumState {
    /// list of allowed origins
    pub allowed_origins: Vec<HeaderValue>,

    /// list of allowed headers
    pub allowed_headers: Vec<HeaderName>,

    /// list of allowed methods
    pub allowed_methods: Vec<Method>,

    pub static_file_dir: Option<String>,

    /// Built-in HTTP body extraction configuration
    pub body_config: BodyConfig,

    #[cfg(feature = "static")]
    pub allowed_static_media_extensions: Vec<String>,
}

impl Debug for FoxtiveAxumState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("foxtive axum state")
    }
}
