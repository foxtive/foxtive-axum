use crate::error::HttpError;
use axum::response::Response;

pub(crate) mod kernel;
pub mod response;
pub mod extractors;

pub type HttpResult = Result<Response, HttpError>;
