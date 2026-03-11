use crate::contracts::ResponseCodeContract;
use crate::enums::response_code::ResponseCode;
use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ext::StructResponseExt;
use axum::response::Response;
use serde::Serialize;

impl<T: Serialize> StructResponseExt for T {
    fn into_response(self) -> Response {
        Responder::send(self, ResponseCode::Ok)
    }

    fn respond_code<C: ResponseCodeContract, M: Into<String>>(self, code: C, msg: M) -> HttpResult {
        Ok(Responder::send_msg(self, code, msg))
    }

    fn respond_msg(self, msg: impl Into<String>) -> HttpResult {
        Ok(Responder::send_msg(self, ResponseCode::Ok, msg))
    }

    fn respond(self) -> HttpResult {
        Ok(Responder::send(self, ResponseCode::Ok))
    }
}
