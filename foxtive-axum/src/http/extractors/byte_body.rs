use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::debug;

#[derive(Debug)]
pub enum ByteExtractionError {
    InvalidUtf8(std::string::FromUtf8Error),
    PayloadTooLarge,
    Other(String),
}

impl IntoResponse for ByteExtractionError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ByteExtractionError::InvalidUtf8(err) => {
                (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", err))
            }
            ByteExtractionError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload too large".to_string(),
            ),
            ByteExtractionError::Other(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };

        (status, error_message).into_response()
    }
}

impl From<std::string::FromUtf8Error> for ByteExtractionError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ByteExtractionError::InvalidUtf8(err)
    }
}

/// Extractor for reading the request body as raw bytes (Vec<u8>).
///
/// # Example
/// ```
/// use foxtive_axum::http::extractors::ByteBody;
///
/// async fn handler(body: ByteBody) -> String {
///     format!("{} bytes received", body.len())
/// }
/// ```
pub struct ByteBody {
    bytes: Vec<u8>,
}

impl ByteBody {
    /// Returns a reference to the raw byte buffer.
    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    /// Consumes the ByteBody and returns the inner buffer.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns the number of bytes in the buffer.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Tries to interpret the bytes as a UTF-8 string.
    pub fn as_utf8(&self) -> Result<String, ByteExtractionError> {
        String::from_utf8(self.bytes.clone()).map_err(ByteExtractionError::from)
    }
}

impl From<Vec<u8>> for ByteBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl From<&[u8]> for ByteBody {
    fn from(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }
}

impl<S> FromRequest<S> for ByteBody
where
    S: Send + Sync,
{
    type Rejection = ByteExtractionError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the body bytes
        let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|err| ByteExtractionError::Other(format!("Failed to read body: {}", err)))?;

        debug!("[byte-body] {} bytes", bytes.len());

        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_and_into_bytes() {
        let raw = b"hello world".to_vec();
        let bb = ByteBody::from(raw.clone());
        assert_eq!(bb.bytes(), &raw);

        let bb = ByteBody::from(&raw[..]);
        assert_eq!(bb.bytes(), &raw);

        let moved = bb.into_bytes();
        assert_eq!(moved, raw);
    }

    #[test]
    fn test_len_and_is_empty() {
        let empty = ByteBody::from(Vec::new());
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let s = ByteBody::from(b"abcd".to_vec());
        assert!(!s.is_empty());
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn test_as_utf8_success() {
        let text = "Some UTF8 👍 data";
        let bb = ByteBody::from(text.as_bytes());
        let utf8 = bb.as_utf8();
        assert!(utf8.is_ok());
        assert_eq!(utf8.unwrap(), text.to_string());
    }

    #[test]
    fn test_as_utf8_failure() {
        // Invalid UTF-8 (continuation byte without leading byte)
        let bytes = vec![0xff, 0xfe, 0xfd];
        let bb = ByteBody::from(bytes);
        let utf8 = bb.as_utf8();
        assert!(utf8.is_err());

        // Verify it's the correct error type
        if let Err(ByteExtractionError::InvalidUtf8(_)) = utf8 {
            // Expected error type
        } else {
            panic!("Expected InvalidUtf8 error");
        }
    }
}
