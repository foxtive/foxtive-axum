use crate::contracts::ResponseCodeContract;
use crate::enums::response_code::ResponseCode;
use crate::helpers::json_message::JsonMessage;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub struct Responder;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse<T: Serialize> {
    pub code: String,
    pub success: bool,
    pub timestamp: u64,
    pub message: Option<String>,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct SeJsonResponse<T> {
    pub code: String,
    pub success: bool,
    pub timestamp: u64,
    pub message: Option<String>,
    pub data: T,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DeJsonResponse<T> {
    pub code: String,
    pub success: bool,
    pub timestamp: u64,
    pub message: Option<String>,
    pub data: T,
}

impl<T: Serialize> Display for JsonResponse<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(serde_json::to_string(self).unwrap().as_str())
    }
}

#[allow(dead_code)]
impl Responder {
    pub fn send_msg<C, D>(data: D, code: C, msg: &str) -> Response
    where
        C: ResponseCodeContract,
        D: Serialize,
    {
        Self::respond(
            JsonMessage::make(data, code.code(), code.success(), Some(msg.to_string())),
            code.status(),
        )
    }

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

    pub fn ok_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::Ok)
    }

    pub fn success_message(msg: &str) -> Response {
        Self::ok_message(msg)
    }

    pub fn warning_message(msg: &str) -> Response {
        Self::bad_req_message(msg)
    }

    pub fn bad_req_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::BadRequest)
    }

    pub fn not_found_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::NotFound)
    }

    pub fn entity_not_found_message(entity: &str) -> Response {
        let msg = format!("Such {entity} does not exists");
        Self::not_found_message(&msg)
    }

    pub fn internal_server_error_message(msg: &str) -> Response {
        Self::message(msg, ResponseCode::InternalServerError)
    }

    pub fn not_found() -> Response {
        Self::not_found_message("Not Found")
    }

    pub fn internal_server_error() -> Response {
        Self::internal_server_error_message("Internal Server Error")
    }

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

    pub fn redirect(url: &'static str) -> Response {
        Redirect::to(url).into_response()
    }

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
