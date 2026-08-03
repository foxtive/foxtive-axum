use crate::contracts::ResponseCodeContract;
use crate::enums::response_code::ResponseCode;
use crate::helpers::json_message::JsonMessage;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Builder for standardized JSON responses.
///
/// All methods return [`Response`] with a consistent JSON envelope containing
/// `code`, `success`, `timestamp`, `message`, and `data` fields.
///
/// # Example
/// ```rust
/// use foxtive_axum::http::responder::Responder;
///
/// let response = Responder::ok_message("Operation completed");
/// ```
pub struct Responder;

/// A standardized JSON response envelope (serializable).
///
/// Contains `code`, `success`, `timestamp`, `message`, and `data` fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse<T: Serialize> {
    /// Application-specific response code.
    pub code: String,
    /// Whether the operation was successful.
    pub success: bool,
    /// Unix timestamp of the response.
    pub timestamp: u64,
    /// Optional human-readable message.
    pub message: Option<String>,
    /// The response payload.
    pub data: T,
}

/// A standardized JSON response envelope (serializable, alias).
#[derive(Debug, Serialize)]
pub struct SeJsonResponse<T> {
    /// Application-specific response code.
    pub code: String,
    /// Whether the operation was successful.
    pub success: bool,
    /// Unix timestamp of the response.
    pub timestamp: u64,
    /// Optional human-readable message.
    pub message: Option<String>,
    /// The response payload.
    pub data: T,
}

/// A standardized JSON response envelope (deserializable, alias).
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DeJsonResponse<T> {
    /// Application-specific response code.
    pub code: String,
    /// Whether the operation was successful.
    pub success: bool,
    /// Unix timestamp of the response.
    pub timestamp: u64,
    /// Optional human-readable message.
    pub message: Option<String>,
    /// The response payload.
    pub data: T,
}

impl<T: Serialize> Display for JsonResponse<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(serde_json::to_string(self).unwrap().as_str())
    }
}

#[allow(dead_code)]
impl Responder {
    /// Send data with a response code and message.
    pub fn send_msg<C, D>(data: D, code: C, msg: impl Into<String>) -> Response
    where
        C: ResponseCodeContract,
        D: Serialize,
    {
        Self::respond(
            JsonMessage::make(data, code.code(), code.success(), Some(msg.into())),
            code.status(),
        )
    }

    /// Send data with a response code (no message).
    pub fn send<C, D>(data: D, code: C) -> Response
    where
        C: ResponseCodeContract,
        D: Serialize,
    {
        Self::respond(
            JsonMessage::make(data, code.code(), code.success(), None),
            code.status(),
        )
    }

    /// Send a 200 OK response with a message.
    pub fn ok_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::Ok)
    }

    /// Alias for [`ok_message`](Self::ok_message).
    pub fn success_message(msg: &str) -> Response {
        Self::ok_message(msg)
    }

    /// Send a 400 Bad Request response with a message.
    pub fn warning_message(msg: &str) -> Response {
        Self::bad_req_message(msg)
    }

    /// Send a 400 Bad Request response with a message.
    pub fn bad_req_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::BadRequest)
    }

    /// Send a 404 Not Found response with a message.
    pub fn not_found_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::NotFound)
    }

    /// Send a 404 Not Found response describing a missing entity.
    pub fn entity_not_found_message(entity: &str) -> Response {
        let msg = format!("Such {entity} does not exists");
        Self::not_found_message(&msg)
    }

    /// Send a 500 Internal Server Error response with a message.
    pub fn internal_server_error_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::InternalServerError)
    }

    /// Send a 404 Not Found response with default message.
    pub fn not_found() -> Response {
        Self::not_found_message("Not Found")
    }

    /// Send a 500 Internal Server Error response with default message.
    pub fn internal_server_error() -> Response {
        Self::internal_server_error_message("Internal Server Error")
    }

    /// Send a response with a message and response code.
    pub fn message<C: ResponseCodeContract>(msg: &str, code: C) -> Response {
        let message = JsonMessage::make((), code.code(), code.success(), Some(msg.to_owned()));

        Self::respond(message, code.status())
    }

    /// Send a response without the standard response wrapper
    ///
    /// # Arguments
    ///
    /// * `data`: Any item that implements serde::Serialize
    /// * `status`: A http status code to respond with
    ///
    /// returns: Response<Body>
    ///
    pub fn respond<T: Serialize>(data: T, status: StatusCode) -> Response {
        Self::make_response(data, status)
    }

    /// Send a redirect response to the given URL.
    pub fn redirect(url: &'static str) -> Response {
        Redirect::to(url).into_response()
    }

    /// Send an HTML response with the given status code.
    pub fn html(html: &str, status: StatusCode) -> Response {
        Response::builder()
            .status(status)
            .header("Content-Type", "text/html")
            .body(html.to_string())
            .expect("response builder")
            .into_response()
    }

    fn make_response<T: Serialize>(data: T, status: StatusCode) -> Response {
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&data).unwrap())
            .expect("response builder")
            .into_response()
    }
}
