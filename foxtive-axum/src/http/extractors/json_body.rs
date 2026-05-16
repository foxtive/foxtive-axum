use crate::http::responder::Responder;
use crate::{FOXTIVE_AXUM, FoxtiveAxumExt};
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use std::ops;
use tracing::debug;

#[derive(Debug)]
pub enum JsonExtractionError {
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidJson(serde_json::Error),
    PayloadTooLarge,
    Other(String),
}

impl IntoResponse for JsonExtractionError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            JsonExtractionError::InvalidUtf8(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", err))
            }
            JsonExtractionError::InvalidJson(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", err))
            }
            JsonExtractionError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload too large".to_string(),
            ),
            JsonExtractionError::Other(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };

        Responder::message(&error_message, status)
    }
}

impl From<std::string::FromUtf8Error> for JsonExtractionError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        JsonExtractionError::InvalidUtf8(err)
    }
}

impl From<serde_json::Error> for JsonExtractionError {
    fn from(err: serde_json::Error) -> Self {
        JsonExtractionError::InvalidJson(err)
    }
}

/// A wrapper struct that holds both the raw JSON string and the deserialized data.
///
/// This struct is useful when you need both the raw JSON string and the parsed
/// object, avoiding multiple deserialization operations.
pub struct JsonBody<T: DeserializeOwned> {
    json: String,
    value: T,
}

impl<T: DeserializeOwned> JsonBody<T> {
    /// Returns a reference to the raw JSON string.
    pub fn body(&self) -> &str {
        &self.json
    }

    /// Consumes the `JsonBody`, returning the raw JSON string.
    pub fn into_body(self) -> String {
        self.json
    }

    /// Returns a reference to the deserialized object.
    pub fn inner(&self) -> &T {
        &self.value
    }

    /// Consumes the `JsonBody`, returning the inner deserialized object.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonExtractionError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // Get max size from JSON body configuration
        let max_size = FOXTIVE_AXUM.app().body_config.json_limit;

        // Extract the body bytes with size limit
        let bytes = axum::body::to_bytes(req.into_body(), max_size)
            .await
            .map_err(|err| {
                if err.to_string().contains("length limit") {
                    JsonExtractionError::PayloadTooLarge
                } else {
                    JsonExtractionError::Other(format!("Failed to read body: {}", err))
                }
            })?;

        // Convert bytes to UTF-8 string efficiently
        let json = String::from_utf8(bytes.to_vec()).map_err(JsonExtractionError::InvalidUtf8)?;

        debug!("[json-body] {}", json);

        // Deserialize JSON string to target type
        let value = serde_json::from_str::<T>(&json).map_err(JsonExtractionError::InvalidJson)?;

        Ok(JsonBody { json, value })
    }
}

impl<T: DeserializeOwned> ops::Deref for JsonBody<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: DeserializeOwned> ops::DerefMut for JsonBody<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str.clone(),
            value,
        };

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*json_body.inner(), expected);
    }

    #[test]
    fn test_into_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str.clone(),
            value,
        };

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(json_body.into_inner(), expected);
    }

    #[test]
    fn test_body() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str.clone(),
            value,
        };

        assert_eq!(json_body.body(), &json_str);
    }

    #[test]
    fn test_into_body() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str.clone(),
            value,
        };

        assert_eq!(json_body.into_body(), json_str);
    }

    #[test]
    fn test_deserialize_success() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str,
            value,
        };

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*json_body.inner(), expected);
    }

    #[test]
    fn test_deserialize_failure() {
        let json_str = r#"{"field1": "value1", "field2": "invalid_int"}"#.to_string();
        let result = serde_json::from_str::<TestStruct>(&json_str);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_to_map() {
        let json_str = r#"{"key1": "value1", "key2": "value2"}"#.to_string();
        let value: HashMap<String, String> = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str,
            value,
        };

        let expected = {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        };

        assert_eq!(*json_body.inner(), expected);
    }

    #[test]
    fn test_deref() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let json_body = JsonBody {
            json: json_str,
            value: value.clone(),
        };

        assert_eq!(*json_body, value);
        assert_eq!(json_body.field1, "value1");
        assert_eq!(json_body.field2, 42);
    }

    #[test]
    fn test_deref_mut() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let value: TestStruct = serde_json::from_str(&json_str).unwrap();
        let mut json_body = JsonBody {
            json: json_str,
            value,
        };

        json_body.field2 = 100;
        assert_eq!(json_body.field2, 100);
    }
}
