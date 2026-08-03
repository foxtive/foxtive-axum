//! Response extension traits, error helpers, and view rendering.
//!
//! Provides extension traits that add `.respond()`, `.respond_msg()`, and
//! `.respond_code()` methods to common types for ergonomic handler responses.

/// Error helpers and the [`error_helpers::ResponseError`] wrapper.
pub mod error_helpers;
/// Extension traits for converting values into standardized HTTP responses.
pub mod ext;
mod message;
/// Response implementations for `AppResult<T>` and `Result<AppResult<T>, JoinError>`.
pub mod respond;
/// Response implementations for `AppResult<T>` via [`ResultResponseExt`].
pub mod result;
/// Response implementations for any `Serialize` type via [`StructResponseExt`].
pub mod r#struct;
#[cfg(feature = "templating")]
mod view;

#[cfg(feature = "templating")]
pub use view::View;

#[cfg(feature = "templating")]
pub use foxtive::TemplateContext as ViewContext;
