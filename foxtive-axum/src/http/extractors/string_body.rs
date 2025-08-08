use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::debug;

#[derive(Debug)]
pub enum StringExtractionError {
    InvalidUtf8(std::string::FromUtf8Error),
    ParseError(String),
    PayloadTooLarge,
    Other(String),
}

impl IntoResponse for StringExtractionError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            StringExtractionError::InvalidUtf8(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", err))
            }
            StringExtractionError::ParseError(err) => {
                (StatusCode::BAD_REQUEST, format!("Parse error: {}", err))
            }
            StringExtractionError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload too large".to_string(),
            ),
            StringExtractionError::Other(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };

        (status, error_message).into_response()
    }
}

impl From<std::string::FromUtf8Error> for StringExtractionError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        StringExtractionError::InvalidUtf8(err)
    }
}

/// Extractor for reading the request body as a plain UTF-8 string.
///
/// # Example
/// ```
/// use foxtive_axum::http::extractors::StringBody;
///
/// async fn handler(body: StringBody) -> String {
///     format!("Received: {}", body.body())
/// }
/// ```
pub struct StringBody {
    body: String,
}

impl StringBody {
    /// Returns a reference to the underlying string body.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Consumes the `StringBody`, returning the inner string.
    pub fn into_body(self) -> String {
        self.body
    }

    /// Returns the length of the string body in bytes.
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// Returns true if the string body is empty.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Tries to parse the body to a specific type that implements `FromStr`.
    /// Returns a result or an error if parsing fails.
    pub fn parse<T: std::str::FromStr>(&self) -> Result<T, StringExtractionError>
    where
        <T as std::str::FromStr>::Err: ToString,
    {
        self.body
            .parse::<T>()
            .map_err(|e| StringExtractionError::ParseError(e.to_string()))
    }
}

impl From<String> for StringBody {
    fn from(body: String) -> Self {
        Self { body }
    }
}

impl From<&str> for StringBody {
    fn from(body: &str) -> Self {
        Self {
            body: body.to_owned(),
        }
    }
}

impl<S> FromRequest<S> for StringBody
where
    S: Send + Sync,
{
    type Rejection = StringExtractionError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the body bytes
        let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|err| StringExtractionError::Other(format!("Failed to read body: {}", err)))?;

        // Convert bytes to UTF-8 string
        let raw = String::from_utf8(bytes.to_vec())?;
        debug!("[string-body] {}", raw);

        Ok(Self { body: raw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_and_into_body() {
        let data = "hello string body".to_string();
        let sb = StringBody::from(data.clone());
        assert_eq!(sb.body(), &data);

        let sb = StringBody::from(&data[..]);
        assert_eq!(sb.body(), &data);

        let moved = sb.into_body();
        assert_eq!(moved, data);
    }

    #[test]
    fn test_len_and_is_empty() {
        let empty = StringBody::from("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let s = StringBody::from("abcde");
        assert!(!s.is_empty());
        assert_eq!(s.len(), 5);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_success() {
        let s = StringBody::from("42");
        let val: i32 = s.parse().unwrap();
        assert_eq!(val, 42);

        let s = StringBody::from("3.1415");
        let val: f64 = s.parse().unwrap();
        assert!((val - 3.1415).abs() < 1e-6);
    }

    #[test]
    fn test_parse_failure() {
        let s = StringBody::from("not_a_number");
        let result: Result<i32, StringExtractionError> = s.parse();
        assert!(result.is_err());

        // Verify it's the correct error type
        if let Err(StringExtractionError::ParseError(msg)) = result {
            // Message should include 'invalid digit' for i32::FromStr
            assert!(msg.to_lowercase().contains("invalid"));
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_deprecated_raw() {
        let data = "raw string body".to_string();
        let sb = StringBody::from(data.clone());
        assert_eq!(sb.body(), &data);
    }

    #[test]
    fn test_parse_bool() {
        let s = StringBody::from("true");
        let val: bool = s.parse().unwrap();
        assert!(val);

        let s = StringBody::from("false");
        let val: bool = s.parse().unwrap();
        assert!(!val);

        let s = StringBody::from("not_a_bool");
        let result: Result<bool, StringExtractionError> = s.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_utf8_emoji() {
        let emoji_text = "Hello 👋 World 🌍";
        let sb = StringBody::from(emoji_text);
        assert_eq!(sb.body(), emoji_text);
        assert_eq!(sb.len(), emoji_text.len()); // Note: this is byte length, not character count
    }
}
