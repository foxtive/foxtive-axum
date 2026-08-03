//! # Foxtive Axum
//!
//! A Rust web framework built on top of [Axum](https://github.com/tokio-rs/axum)
//! providing standardized response formats, error handling, custom extractors,
//! and utilities for building REST APIs.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use axum::routing::get;
//! use axum::Router;
//! use foxtive::results::AppResult;
//! use foxtive::setup::trace::Tracing;
//! use foxtive_axum::http::response::ext::StructResponseExt;
//! use foxtive_axum::http::HttpResult;
//! use foxtive_axum::server::Server;
//! use std::sync::Arc;
//! use foxtive::App;
//!
//! #[tokio::main]
//! async fn main() -> AppResult<()> {
//!     let app = App::builder("MyApp", "MYAPP")
//!         .build()
//!         .await?;
//!
//!     let router = Router::new().route("/", get(handler));
//!
//!     Server::new(app)
//!         .host("127.0.0.1")
//!         .port(3000)
//!         .router(router)
//!         .tracing(Tracing::default())
//!         .run()
//!         .await
//! }
//!
//! async fn handler() -> HttpResult {
//!     "Hello, World!".respond()
//! }
//! ```

#![warn(missing_docs)]

/// Trait contracts for response codes and other abstractions.
pub mod contracts;
/// Enumerations for response codes and status mapping.
pub mod enums;
/// HTTP error types with automatic response conversion.
pub mod error;
/// Helper utilities for building JSON responses.
pub mod helpers;
/// HTTP kernel, extractors, response helpers, and middleware.
pub mod http;
/// Server configuration and lifecycle management.
pub mod server;
/// Testing utilities for integration tests.
pub mod testing;

// Re-export core foxtive types for convenience
pub use foxtive::App;
pub use foxtive::AppBuilder;
pub use std::sync::Arc;
