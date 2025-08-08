use crate::helpers::responder::Responder;
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

/// A wrapper struct that holds the deserialized data.
///
/// This struct is useful when you need the parsed json data
/// object, avoiding multiple deserialization operations.
pub struct JsonBody<T: DeserializeOwned>(T);

impl<T: DeserializeOwned> JsonBody<T> {
    /// Creates a new `JsonBody` instance by parsing the given JSON string.
    ///
    /// # Arguments
    /// * `json` - A string slice containing valid JSON
    ///
    /// # Returns
    /// * `Result<JsonBody<T>, JsonExtractionError>` - Result containing the new instance or an error
    ///
    /// # Errors
    /// Returns an error if the JSON string cannot be deserialized into the target type T.
    pub fn new(json: String) -> Result<JsonBody<T>, JsonExtractionError> {
        Ok(JsonBody(serde_json::from_str::<T>(&json)?))
    }

    /// Returns a reference to the deserialized object.
    ///
    /// # Example
    /// ```
    /// use foxtive_axum::http::extractors::JsonBody;
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let manual_body = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();
    /// let de_json_body = JsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.inner(), &manual_body);
    /// ```
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Consumes the `JsonBody`, returning the inner deserialized object.
    ///
    /// # Example
    /// ```
    /// use foxtive_axum::http::extractors::JsonBody;
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let manual_body = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();
    /// let de_json_body = JsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.into_inner(), manual_body);
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonExtractionError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the body bytes
        let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|err| JsonExtractionError::Other(format!("Failed to read body: {}", err)))?;

        // Convert bytes to UTF-8 string
        let raw = String::from_utf8(bytes.to_vec())?;

        debug!("[json-body] {}", raw);

        Self::new(raw)
    }
}

impl<T: DeserializeOwned> ops::Deref for JsonBody<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: DeserializeOwned> ops::DerefMut for JsonBody<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = JsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*de_json_body.inner(), expected);
    }

    #[test]
    fn test_into_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = JsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(de_json_body.into_inner(), expected);
    }

    #[test]
    fn test_deserialize_success() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = JsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*de_json_body.inner(), expected);
    }

    #[test]
    fn test_deserialize_failure() {
        let json_str = r#"{"field1": "value1", "field2": "invalid_int"}"#.to_string();
        let result = JsonBody::<TestStruct>::new(json_str);

        assert!(result.is_err());
        if let Err(JsonExtractionError::InvalidJson(_)) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidJson error");
        }
    }

    #[test]
    fn test_deserialize_to_map() {
        let json_str = r#"{"key1": "value1", "key2": "value2"}"#.to_string();
        let de_json_body = JsonBody::<HashMap<String, String>>::new(json_str).unwrap();

        let expected = {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        };

        assert_eq!(*de_json_body.inner(), expected);
    }
}
