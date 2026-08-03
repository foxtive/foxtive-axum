use crate::contracts::ResponseCodeContract;
use crate::http::HttpResult;
use axum::http::StatusCode;
use axum::response::Response;

/// Extension trait for sending `Result` types as HTTP responses.
///
/// Implemented for `AppResult<T>` where `T: Serialize`.
pub trait ResultResponseExt {
    /// Send the result data with the given response code.
    fn send_result<C: ResponseCodeContract>(self, code: C) -> HttpResult;

    /// Send the result data with the given response code and message.
    fn send_result_msg<C: ResponseCodeContract, M: Into<String>>(
        self,
        code: C,
        msg: M,
    ) -> HttpResult;
}

/// Extension trait for converting `AppMessage` into an HTTP response.
pub trait AppMessageExt {
    /// Convert the message into an HTTP response.
    ///
    /// Success messages produce `Ok(Response)`, error messages produce `Err(HttpError)`.
    fn respond(self) -> HttpResult;
}

/// Extension trait for responding with HTML strings.
pub trait HtmlResponderExt {
    /// Render as an HTML response with 200 OK status.
    fn respond(self) -> HttpResult;

    /// Render as an HTML response with a custom status code.
    fn respond_status(self, status: StatusCode) -> HttpResult;
}

/// Extension trait for responding with serializable data and response codes.
///
/// Implemented for `AppResult<T>` and `Result<AppResult<T>, JoinError>`.
pub trait ResponderExt {
    /// Respond with data, a message, and a response code.
    fn respond_code<C: ResponseCodeContract, M: Into<String>>(self, msg: M, code: C) -> HttpResult;

    /// Respond with data and a success message (200 OK).
    fn respond_msg(self, suc: impl Into<String>) -> HttpResult;

    /// Respond with data using 200 OK.
    fn respond(self) -> HttpResult;
}

/// Extension trait for responding with any `Serialize` type.
///
/// Implemented as a blanket impl for all `Serialize` types.
///
/// # Example
/// ```rust
/// use foxtive_axum::http::response::ext::StructResponseExt;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User { name: String }
///
/// let user = User { name: "Alice".into() };
/// let result = user.respond();
/// ```
pub trait StructResponseExt: Sized {
    /// Convert into a standalone JSON `Response` (200 OK, no envelope message).
    fn into_response(self) -> Response;

    /// Respond with data, a response code, and a message.
    fn respond_code<C: ResponseCodeContract, M: Into<String>>(self, code: C, msg: M) -> HttpResult;

    /// Respond with data and a message (200 OK).
    fn respond_msg(self, msg: impl Into<String>) -> HttpResult;

    /// Respond with data (200 OK).
    fn respond(self) -> HttpResult;
}

/// Extension trait for `AppResult<T>` with empty-check helpers.
pub trait OptionResultResponseExt<T> {
    /// Returns `true` if the result is a NotFound error.
    fn is_empty(&self) -> bool;

    /// Returns `true` if the result is an error.
    fn is_error(&self) -> bool;

    /// Returns `true` if the result is an error or a NotFound.
    fn is_error_or_empty(&self) -> bool;

    /// Send the result as a response with the given code and message.
    fn send_response<C: ResponseCodeContract, M: Into<String>>(self, code: C, msg: M)
    -> HttpResult;
}

/// Extension trait for converting arbitrary values into `HttpResult`.
pub trait IntoHttpResultExt {
    /// Convert `self` into an `HttpResult`.
    fn http_result(self) -> HttpResult;
}
