use crate::http::response::anyhow::helpers::make_status_code;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use foxtive::Error;
use foxtive::prelude::AppMessage;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    Std(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("{0}")]
    AppError(#[from] Error),
    #[error("{0}")]
    AppMessage(#[from] AppMessage),
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[cfg(feature = "validator")]
    #[error("Validation Error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}

impl HttpError {
    pub fn into_app_error(self) -> foxtive::Error {
        foxtive::Error::from(self)
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            HttpError::AppError(e) => make_status_code(e),
            HttpError::AppMessage(m) => m.status_code(),
            #[cfg(feature = "validator")]
            HttpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_error_response(self) -> Response {
        helpers::make_response(&self.into_app_error())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HttpError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        HttpError::Std(error)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        helpers::make_http_error_response(&self)
    }
}

pub(crate) mod helpers {
    use crate::error::HttpError;
    pub(crate) use crate::http::response::anyhow::helpers::make_response;
    use axum::response::Response;
    use foxtive::prelude::AppMessage;
    use tracing::error;

    pub(crate) fn make_http_error_response(err: &HttpError) -> Response {
        #[cfg(feature = "validator")]
        use crate::enums::response_code::ResponseCode;
        #[cfg(feature = "validator")]
        use crate::http::responder::Responder;

        match err {
            HttpError::AppMessage(m) => make_response(&m.clone().ae()),
            HttpError::AppError(e) => make_response(e),
            #[cfg(feature = "validator")]
            HttpError::ValidationError(e) => {
                error!("Validation Error: {e}");
                Responder::send_msg(e.errors(), ResponseCode::BadRequest, "Validation Error")
            }
            _ => {
                error!("Error: {err}");
                make_response(&foxtive::Error::from(AppMessage::InternalServerError))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::helpers::make_http_error_response;
    use foxtive::Error;

    #[test]
    fn test_app_error() {
        let error = HttpError::AppError(Error::from(AppMessage::InternalServerError));
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_app_message() {
        let error = HttpError::AppMessage(AppMessage::InternalServerError);
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_std_error() {
        #[allow(clippy::io_other_error)]
        let error = HttpError::Std(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test",
        )));
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_validation_error() {
        let error = HttpError::ValidationError(validator::ValidationErrors::new());
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 400);
    }
}
