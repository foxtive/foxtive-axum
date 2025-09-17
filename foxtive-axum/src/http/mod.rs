use crate::error::HttpError;
use axum::response::Response;

pub mod extractors;
pub(crate) mod kernel;
pub mod response;
pub mod responder;

pub type HttpResult = Result<Response, HttpError>;
