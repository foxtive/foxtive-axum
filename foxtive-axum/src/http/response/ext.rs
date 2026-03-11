use crate::contracts::ResponseCodeContract;
use crate::http::HttpResult;
use axum::http::StatusCode;
use axum::response::Response;

pub trait ResultResponseExt {
    fn send_result<C: ResponseCodeContract>(self, code: C) -> HttpResult;

    fn send_result_msg<C: ResponseCodeContract, M: Into<String>>(
        self,
        code: C,
        msg: M,
    ) -> HttpResult;
}

pub trait AppMessageExt {
    fn respond(self) -> HttpResult;
}

pub trait HtmlResponderExt {
    fn respond(self) -> HttpResult;

    fn respond_status(self, status: StatusCode) -> HttpResult;
}

pub trait ResponderExt {
    fn respond_code<C: ResponseCodeContract, M: Into<String>>(self, msg: M, code: C) -> HttpResult;

    fn respond_msg(self, suc: impl Into<String>) -> HttpResult;

    fn respond(self) -> HttpResult;
}

pub trait StructResponseExt: Sized {
    fn into_response(self) -> Response;

    fn respond_code<C: ResponseCodeContract, M: Into<String>>(self, code: C, msg: M) -> HttpResult;

    fn respond_msg(self, msg: impl Into<String>) -> HttpResult;

    fn respond(self) -> HttpResult;
}

pub trait OptionResultResponseExt<T> {
    fn is_empty(&self) -> bool;

    fn is_error(&self) -> bool;

    fn is_error_or_empty(&self) -> bool;

    fn send_response<C: ResponseCodeContract, M: Into<String>>(self, code: C, msg: M)
    -> HttpResult;
}

pub trait IntoHttpResultExt {
    fn http_result(self) -> HttpResult;
}
