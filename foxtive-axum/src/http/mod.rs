//! HTTP kernel, extractors, response helpers, and middleware.
//!
//! This module contains the core HTTP infrastructure:
//! - **Extractors**: [`extractors::JsonBody`], [`extractors::StringBody`], [`extractors::ByteBody`]
//! - **Response traits**: [`response::ext`] extension traits for ergonomic responding
//! - **Responder**: [`responder::Responder`] for building standardized JSON responses

use crate::error::HttpError;
use axum::response::Response;

/// Custom request body extractors with size-limit enforcement.
pub mod extractors;
pub(crate) mod kernel;
/// Standardized JSON response builder.
pub mod responder;
/// Response extension traits and error helpers.
pub mod response;
#[cfg(feature = "static")]
pub(crate) mod static_file;

/// A convenience type alias for `Result<Response, HttpError>`.
///
/// Use this as the return type of your axum handlers.
pub type HttpResult = Result<Response, HttpError>;
