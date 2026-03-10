use crate::contracts::ResponseCodeContract;
use crate::enums::response_code::ResponseCode;
use crate::error::HttpError;
use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ext::AppMessageExt;
use foxtive::prelude::AppMessage;
use foxtive::results::AppResult;

impl AppMessageExt for AppMessage {
    fn respond(self) -> HttpResult {
        let status = self.status_code();
        match status.is_success() {
            true => Ok(Responder::message(
                &self.message(),
                ResponseCode::from_status(self.status_code()),
            )),
            false => Err(HttpError::AppMessage(self)),
        }
    }
}

impl AppMessageExt for AppResult<AppMessage> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => Err(HttpError::AppError(err)),
        }
    }
}

impl AppMessageExt for Result<AppMessage, AppMessage> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => err.respond(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::response::ext::AppMessageExt;
    use foxtive::Error;
    use foxtive::prelude::AppMessage;

    #[test]
    fn test_app_message_respond_success() {
        let msg = AppMessage::success("Yes");
        let result = msg.respond();
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_message_respond_error() {
        let msg = AppMessage::internal_server_error("Internal Server Error");
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_respond() {
        let msg: Result<AppMessage, Error> =
            Ok(AppMessage::internal_server_error("Internal Server Error"));
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_error_respond() {
        let msg = Err(AppMessage::internal_server_error("Internal Server Error"));
        let result = msg.respond();
        assert!(result.is_err());
    }
}
