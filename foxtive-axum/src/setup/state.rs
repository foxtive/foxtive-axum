use axum::http::{HeaderName, HeaderValue, Method};
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct FoxtiveAxumState {
    /// list of allowed origins
    pub allowed_origins: Vec<HeaderValue>,

    /// list of allowed headers
    pub allowed_headers: Vec<HeaderName>,

    /// list of allowed methods
    pub allowed_methods: Vec<Method>,
}

impl Debug for FoxtiveAxumState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("foxtive axum state")
    }
}
