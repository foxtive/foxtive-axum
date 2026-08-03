use axum::http::StatusCode;
use std::str::FromStr;

/// A trait that maps application-level response codes to HTTP status codes.
///
/// Implementors provide a string code, an HTTP status, and a success indicator.
/// The default [`success`](ResponseCodeContract::success) method considers 2xx
/// status codes as successful.
pub trait ResponseCodeContract: Clone {
    /// Returns the application-specific response code string (e.g. "000").
    fn code(&self) -> &str;

    /// Returns the corresponding HTTP status code.
    fn status(&self) -> StatusCode;

    /// Returns `true` if the HTTP status indicates success (2xx).
    fn success(&self) -> bool {
        let code = self.status().as_u16();
        (200..300).contains(&code)
    }

    /// Creates a response code from its string representation.
    fn from_code(code: &str) -> Self;

    /// Creates a response code from an HTTP status code.
    fn from_status(status: StatusCode) -> Self;
}

impl ResponseCodeContract for StatusCode {
    fn code(&self) -> &str {
        self.as_str()
    }

    fn status(&self) -> StatusCode {
        *self
    }

    fn from_code(code: &str) -> Self {
        StatusCode::from_str(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn from_status(status: StatusCode) -> Self {
        status
    }
}
