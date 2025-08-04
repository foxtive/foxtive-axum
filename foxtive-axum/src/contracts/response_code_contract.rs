use axum::http::StatusCode;
use std::str::FromStr;

pub trait ResponseCodeContract: Clone {
    fn code(&self) -> &str;

    fn status(&self) -> StatusCode;

    fn success(&self) -> bool {
        let code = self.status().as_u16();
        (200..300).contains(&code)
    }

    fn from_code(code: &str) -> Self;

    fn from_status(status: StatusCode) -> Self;
}

impl ResponseCodeContract for StatusCode {
    fn code(&self) -> &str {
        self.as_str()
    }

    fn status(&self) -> StatusCode {
        *self
    }

    fn from_code(code: &str) -> Self {
        StatusCode::from_str(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn from_status(status: StatusCode) -> Self {
        status
    }
}
