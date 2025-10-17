use crate::contracts::ResponseCodeContract;
use crate::enums::response_code::ResponseCode;
use crate::error::HttpError;
use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ext::{HtmlResponderExt, ResponderExt, ResultResponseExt};
use axum::http::StatusCode;
use foxtive::prelude::AppResult;
use serde::Serialize;
use tokio::task::JoinError;

impl<T> ResponderExt for AppResult<T>
where
    T: Sized + Serialize,
{
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult {
        self.send_result_msg(code, msg)
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        self.send_result_msg(ResponseCode::Ok, msg)
    }

    fn respond(self) -> HttpResult {
        self.send_result(ResponseCode::Ok)
    }
}

impl<T> ResponderExt for Result<AppResult<T>, JoinError>
where
    T: Sized + Serialize,
{
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult {
        match self {
            Ok(val) => val.send_result_msg(code, msg),
            Err(err) => Err(HttpError::AppError(err.into())),
        }
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        match self {
            Ok(val) => val.send_result_msg(ResponseCode::Ok, msg),
            Err(err) => Err(HttpError::AppError(err.into())),
        }
    }

    fn respond(self) -> HttpResult {
        match self {
            Ok(val) => val.send_result(ResponseCode::Ok),
            Err(err) => Err(HttpError::AppError(err.into())),
        }
    }
}

impl HtmlResponderExt for &str {
    fn respond(self) -> HttpResult {
        Ok(Responder::html(self, StatusCode::OK))
    }

    fn respond_status(self, status: StatusCode) -> HttpResult {
        Ok(Responder::html(self, status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::json;
    use tokio::task;

    #[test]
    fn test_app_result_respond_code() {
        let data = json!({"key": "value"});
        let app_result: AppResult<_> = Ok(data.clone());

        let result = app_result.respond_code("Created", ResponseCode::Created);
        match result {
            Ok(response) => {
                assert_eq!(response.status(), StatusCode::CREATED);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }

    #[test]
    fn test_join_result_respond_ok() {
        let data = json!({"key": "value"});
        let result: Result<AppResult<_>, JoinError> = Ok(Ok(data.clone()));

        let response = result.respond();
        match response {
            Ok(response) => assert_eq!(response.status(), StatusCode::OK),
            Err(e) => panic!("Expected Ok, got Err: {e:?}"),
        }
    }

    #[test]
    fn test_join_result_respond_msg_ok() {
        let data = json!({"key": "value"});
        let result: Result<AppResult<_>, JoinError> = Ok(Ok(data.clone()));

        let response = result.respond_msg("All good");
        match response {
            Ok(response) => assert_eq!(response.status(), StatusCode::OK),
            Err(e) => panic!("Expected Ok, got Err: {e:?}"),
        }
    }

    #[test]
    fn test_join_result_respond_code_ok() {
        let data = json!({"key": "value"});
        let result: Result<AppResult<_>, JoinError> = Ok(Ok(data.clone()));

        let response = result.respond_code("Accepted", ResponseCode::Accepted);
        match response {
            Ok(response) => assert_eq!(response.status(), StatusCode::ACCEPTED),
            Err(e) => panic!("Expected Ok, got Err: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_join_result_respond_err() {
        let err: JoinError = task::spawn(async {
            panic!("test panic");
        })
        .await
        .unwrap_err();

        let result: Result<AppResult<()>, JoinError> = Err(err);

        let response = result.respond();
        assert!(matches!(response, Err(HttpError::AppError(_))));
    }

    #[tokio::test]
    async fn test_join_result_respond_msg_err() {
        let err: JoinError = task::spawn(async {
            panic!("another panic");
        })
        .await
        .unwrap_err();

        let result: Result<AppResult<()>, JoinError> = Err(err);

        let response = result.respond_msg("Failure");
        assert!(matches!(response, Err(HttpError::AppError(_))));
    }

    #[tokio::test]
    async fn test_join_result_respond_code_err() {
        let err: JoinError = task::spawn(async {
            panic!("yet another panic");
        })
        .await
        .unwrap_err();

        let result: Result<AppResult<()>, JoinError> = Err(err);

        let response = result.respond_code("Failure", ResponseCode::InternalServerError);
        assert!(matches!(response, Err(HttpError::AppError(_))));
    }
}
