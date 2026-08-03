//! HTTP error types for the framework.
//!
//! [`HttpError`] is the primary error type used in handler return values.
//! It implements [`IntoResponse`](axum::response::IntoResponse) so errors
//! are automatically converted to proper HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use foxtive::prelude::AppMessage;
use std::string::FromUtf8Error;
use thiserror::Error;
use tokio::task::JoinError;
use crate::contracts::ResponseCodeContract;

/// HTTP error type that converts application errors into HTTP responses.
///
/// This enum implements [`IntoResponse`], so it can be returned directly
/// from axum handlers. Each variant maps to an appropriate HTTP status code.
///
/// # Example
/// ```rust
/// use foxtive_axum::error::HttpError;
/// use foxtive::prelude::AppMessage;
///
/// let err = HttpError::from(AppMessage::not_found("user"));
/// ```
#[derive(Error, Debug)]
pub enum HttpError {
    /// A boxed standard error (maps to 500).
    #[error("{0}")]
    Std(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// An application-level error message (maps to the message's status code).
    #[error("{0}")]
    AppError(#[from] AppMessage),
    /// A UTF-8 encoding error (maps to 500).
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    /// A tokio task join error (maps to 500).
    #[error("Join Error: {0}")]
    JoinError(#[from] JoinError),
    /// A validation error from the `validator` crate (maps to 400).
    #[cfg(feature = "validator")]
    #[error("Validation Error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}

impl HttpError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            HttpError::AppError(m) => m.status_code(),
            #[cfg(feature = "validator")]
            HttpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HttpError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        HttpError::Std(error)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match &self {
            HttpError::AppError(m) => {
                m.log();
                make_json_error_response(&m.message(), m.status_code())
            }
            #[cfg(feature = "validator")]
            HttpError::ValidationError(e) => {
                use crate::enums::response_code::ResponseCode;
                use crate::http::responder::Responder;
                tracing::error!("Validation Error: {e}");
                Responder::send_msg(e.errors(), ResponseCode::BadRequest, "Validation Error")
            }
            _ => {
                tracing::error!("Error: {self}");
                make_json_error_response(
                    &AppMessage::internal_server_error("Internal Server Error").message(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}

/// Build a JSON error response using the standard response format.
fn make_json_error_response(body: &str, status: StatusCode) -> Response {
    use crate::enums::response_code::ResponseCode;
    use crate::http::responder::Responder;
    let code = ResponseCode::from_status(status);
    Responder::message(body, code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxtive::prelude::AppMessage;

    #[test]
    fn test_app_error() {
        let error = HttpError::AppError(AppMessage::internal_server_error(""));
        let response = error.into_response();
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_std_error() {
        #[allow(clippy::io_other_error)]
        let error = HttpError::Std(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test",
        )));
        let response = error.into_response();
        assert_eq!(response.status(), 500);
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_validation_error() {
        let error = HttpError::ValidationError(validator::ValidationErrors::new());
        let response = error.into_response();
        assert_eq!(response.status(), 400);
    }
}
