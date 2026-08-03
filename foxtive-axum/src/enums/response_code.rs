use crate::contracts::ResponseCodeContract;
use axum::http::StatusCode;

/// Standardized response codes mapping to HTTP status codes.
///
/// Each variant provides a string code (e.g. "000") and a corresponding
/// HTTP status code for use in API responses.
///
/// # Example
/// ```rust
/// use foxtive_axum::enums::response_code::ResponseCode;
/// use foxtive_axum::contracts::ResponseCodeContract;
/// use axum::http::StatusCode;
///
/// assert_eq!(ResponseCode::Ok.status(), StatusCode::OK);
/// assert_eq!(ResponseCode::Ok.code(), "000");
/// assert!(ResponseCode::Ok.success());
/// ```
#[derive(Clone)]
pub enum ResponseCode {
    /// 200 OK - operation succeeded.
    Ok,
    /// 201 Created - resource was created.
    Created,
    /// 202 Accepted - request accepted for processing.
    Accepted,
    /// 204 No Content - success with no response body.
    NoContent,
    /// 400 Bad Request - client sent an invalid request.
    BadRequest,
    /// 401 Unauthorized - authentication is required.
    Unauthorized,
    /// 402 Payment Required - payment is required.
    PaymentRequired,
    /// 403 Forbidden - client lacks permission.
    Forbidden,
    /// 404 Not Found - resource does not exist.
    NotFound,
    /// 409 Conflict - request conflicts with current state.
    Conflict,
    /// 500 Internal Server Error - unexpected server failure.
    InternalServerError,
    /// 503 Service Unavailable - server is temporarily unavailable.
    ServiceUnavailable,
    /// 501 Not Implemented - functionality not implemented.
    NotImplemented,
    /// 405 Method Not Allowed - HTTP method is not allowed for this resource.
    MethodNotAllowed,
}

impl ResponseCodeContract for ResponseCode {
    fn code(&self) -> &str {
        match self {
            ResponseCode::Ok => "000",
            ResponseCode::Created => "001",
            ResponseCode::Accepted => "002",
            ResponseCode::NoContent => "003",
            ResponseCode::BadRequest => "004",
            ResponseCode::Unauthorized => "005",
            ResponseCode::PaymentRequired => "006",
            ResponseCode::Forbidden => "007",
            ResponseCode::NotFound => "008",
            ResponseCode::Conflict => "009",
            ResponseCode::InternalServerError => "010",
            ResponseCode::ServiceUnavailable => "011",
            ResponseCode::NotImplemented => "012",
            ResponseCode::MethodNotAllowed => "013",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            ResponseCode::Ok => StatusCode::OK,
            ResponseCode::Created => StatusCode::CREATED,
            ResponseCode::Accepted => StatusCode::ACCEPTED,
            ResponseCode::NoContent => StatusCode::NO_CONTENT,
            ResponseCode::BadRequest => StatusCode::BAD_REQUEST,
            ResponseCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ResponseCode::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
            ResponseCode::Forbidden => StatusCode::FORBIDDEN,
            ResponseCode::NotFound => StatusCode::NOT_FOUND,
            ResponseCode::Conflict => StatusCode::CONFLICT,
            ResponseCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ResponseCode::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ResponseCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        }
    }

    fn from_code(code: &str) -> Self {
        match code {
            "000" => ResponseCode::Ok,
            "001" => ResponseCode::Created,
            "002" => ResponseCode::Accepted,
            "003" => ResponseCode::NoContent,
            "004" => ResponseCode::BadRequest,
            "005" => ResponseCode::Unauthorized,
            "006" => ResponseCode::PaymentRequired,
            "007" => ResponseCode::Forbidden,
            "008" => ResponseCode::NotFound,
            "009" => ResponseCode::Conflict,
            "010" => ResponseCode::InternalServerError,
            "011" => ResponseCode::ServiceUnavailable,
            "012" => ResponseCode::NotImplemented,
            "013" => ResponseCode::MethodNotAllowed,
            _ => ResponseCode::InternalServerError,
        }
    }

    fn from_status(status: StatusCode) -> Self {
        match status {
            StatusCode::OK => ResponseCode::Ok,
            StatusCode::CREATED => ResponseCode::Created,
            StatusCode::ACCEPTED => ResponseCode::Accepted,
            StatusCode::NO_CONTENT => ResponseCode::NoContent,
            StatusCode::BAD_REQUEST => ResponseCode::BadRequest,
            StatusCode::UNAUTHORIZED => ResponseCode::Unauthorized,
            StatusCode::PAYMENT_REQUIRED => ResponseCode::PaymentRequired,
            StatusCode::FORBIDDEN => ResponseCode::Forbidden,
            StatusCode::NOT_FOUND => ResponseCode::NotFound,
            StatusCode::CONFLICT => ResponseCode::Conflict,
            StatusCode::INTERNAL_SERVER_ERROR => ResponseCode::InternalServerError,
            StatusCode::SERVICE_UNAVAILABLE => ResponseCode::ServiceUnavailable,
            StatusCode::NOT_IMPLEMENTED => ResponseCode::NotImplemented,
            StatusCode::METHOD_NOT_ALLOWED => ResponseCode::MethodNotAllowed,
            _ => ResponseCode::InternalServerError,
        }
    }
}
