use crate::error::HttpError;
use foxtive::Error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ResponseError {
    pub error: foxtive::Error,
}

impl ResponseError {
    pub fn new(error: foxtive::Error) -> Self {
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
        match value {
            HttpError::AppError(e) => ResponseError::new(e),
            HttpError::AppMessage(e) => ResponseError::new(e.into_anyhow()),
            HttpError::Std(e) => ResponseError::new(Error::from_boxed(e)),
            _ => ResponseError::new(foxtive::Error::from(value)),
        }
    }
}

impl From<foxtive::Error> for ResponseError {
    fn from(value: Error) -> Self {
        ResponseError::new(value)
    }
}

pub mod helpers {
    use crate::contracts::ResponseCodeContract;
    use crate::enums::response_code::ResponseCode;
    use crate::error::HttpError;
    use crate::http::responder::Responder;
    use axum::http::StatusCode;
    use axum::response::Response;
    use foxtive::prelude::AppMessage;
    use tracing::error;

    pub fn make_status_code(err: &foxtive::Error) -> StatusCode {
        match err.downcast_ref::<AppMessage>() {
            Some(msg) => msg.status_code(),
            None => match err.downcast_ref::<HttpError>() {
                Some(err) => err.status_code(),
                None => match err.downcast_ref::<serde_json::Error>() {
                    None => StatusCode::INTERNAL_SERVER_ERROR,
                    Some(err) => {
                        error!("Json-Error: {err}");
                        StatusCode::BAD_REQUEST
                    }
                },
            },
        }
    }

    pub fn make_response(err: &foxtive::Error) -> Response {
        let status = make_status_code(err);

        match err.downcast_ref::<AppMessage>() {
            Some(msg) => {
                msg.log();
                make_json_response(&msg.message(), status)
            }
            None => match err.downcast_ref::<HttpError>() {
                Some(err) => crate::error::helpers::make_http_error_response(err),
                None => match err.downcast_ref::<serde_json::Error>() {
                    Some(err) => {
                        error!("Error: {err}");
                        // We can't send JSON error as a response, we don't know what may be leaked
                        make_json_response(
                            "Data processing error",
                            StatusCode::BAD_REQUEST,
                        )
                    }
                    None => {
                        error!("Error: {err}");
                        make_json_response(
                            &AppMessage::internal_server_error("Internal Server Error").message(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }
                },
            },
        }
    }

    pub fn make_json_response(body: &str, status: StatusCode) -> Response {
        let code = ResponseCode::from_status(status);
        Responder::message(body, code)
    }
}
