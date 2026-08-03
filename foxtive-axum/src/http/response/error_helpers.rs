use crate::error::HttpError;
use axum::response::{IntoResponse, Response};
use foxtive::prelude::AppMessage;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

/// A response-compatible error wrapper around [`HttpError`].
///
/// This type implements [`IntoResponse`] so it can be returned directly
/// from axum handlers.
///
/// # Example
/// ```rust
/// use foxtive_axum::http::response::error_helpers::ResponseError;
/// use foxtive::prelude::AppMessage;
///
/// let err = ResponseError::from(AppMessage::not_found("user"));
/// ```
#[derive(Debug, Error)]
pub struct ResponseError {
    /// The underlying HTTP error.
    pub error: HttpError,
}

impl ResponseError {
    /// Create a new `ResponseError` from an `HttpError`.
    pub fn new(error: HttpError) -> Self {
        Self { error }
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<HttpError> for ResponseError {
    fn from(value: HttpError) -> Self {
        ResponseError::new(value)
    }
}

impl From<AppMessage> for ResponseError {
    fn from(value: AppMessage) -> Self {
        ResponseError::new(HttpError::from(value))
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        self.error.into_response()
    }
}

/// Helper functions for building JSON error responses.
pub mod helpers {
    use crate::contracts::ResponseCodeContract;
    use crate::enums::response_code::ResponseCode;
    use crate::http::responder::Responder;
    use axum::http::StatusCode;
    use axum::response::Response;
    use foxtive::prelude::AppMessage;
    use tracing::error;

    /// Build a JSON error response with the given body text and HTTP status.
    pub fn make_json_response(body: &str, status: StatusCode) -> Response {
        let code = ResponseCode::from_status(status);
        Responder::message(body, code)
    }

    /// Build a response from an AppMessage directly.
    pub fn make_response_from_message(msg: &AppMessage) -> Response {
        let status = msg.status_code();
        msg.log();
        make_json_response(&msg.message(), status)
    }

    /// Build a response for a generic error (non-AppMessage).
    pub fn make_error_response(err: &dyn std::error::Error) -> Response {
        error!("Error: {err}");
        make_json_response(
            &AppMessage::internal_server_error("Internal Server Error").message(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}
