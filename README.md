# Foxtive Axum

Foxtive Axum is a Rust web framework built on top of [Axum](https://github.com/tokio-rs/axum) that provides standardized response formats, error handling, custom extractors, and utilities for building REST APIs.

## Features

- Standardized JSON response format
- Integrated error handling with HTTP status codes
- **Custom request body extractors** for enhanced data handling
- CORS support
- Static file serving (optional)
- Request validation (optional)
- Tracing and logging integration
- Panic recovery middleware

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
foxtive-axum = { version = "0.6.0" }
```

### Features

Foxtive Axum comes with optional features:

- `cors` - Enables CORS support
- `static` - Enables static file serving
- `validator` - Enables request validation

To enable features, add them to your `Cargo.toml`:

```toml
[dependencies]
foxtive-axum = { version = "0.1.0", features = ["cors", "static"] }
```

## Usage

### Basic Setup

```rust
use axum::routing::get;
use axum::Router;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::setup::FoxtiveSetup;
use foxtive::Environment;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::http::HttpResult;
use foxtive_axum::server::Server;
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create your routes
    let app = Router::new()
        .route("/", get(handler));

    // Setup Foxtive core
    let foxtive_setup = FoxtiveSetup {
        env_prefix: "FOXTIVE".to_string(),
        private_key: "".to_string(),
        public_key: "".to_string(),
        app_key: "".to_string(),
        app_code: "BASIC".to_string(),
        app_name: "Basic".to_string(),
        env: Environment::Local,
    };

    // Configure & run server
    Server::new(foxtive_setup)
        .host("127.0.0.1")
        .port(3000)
        .router(app)
        .tracing(Tracing::minimal())
        .bootstrap(|_setup| async {
            info!("Bootstrapping application ...");
            Ok(())
        })
        .on_started(|| info!("Server started successfully"))
        .run()
        .await
}

async fn handler() -> HttpResult {
    "Hello, World!".respond()
}
```

## Custom Request Body Extractors

Foxtive Axum provides three powerful custom extractors for handling request bodies with enhanced capabilities:

### `JsonBody<T>` - Enhanced JSON Extractor

An extractor that deserializes JSON while preserving the original raw JSON string. Perfect for logging, forwarding, validation, or when you need both parsed data and the original JSON.

#### Methods

- `body() -> &String` - Get reference to raw JSON string
- `into_body(self) -> String` - Consume and get raw JSON string
- `inner() -> &T` - Get reference to deserialized object
- `into_inner(self) -> T` - Consume and get deserialized object
- Implements `Deref` and `DerefMut` for direct access to `T`

#### Example

```rust
use axum::{routing::post, Router};
use foxtive_axum::http::extractors::JsonBody;
use foxtive_axum::http::ext::StructResponseExt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: Option<u32>,
}

async fn create_user(json: JsonBody<CreateUserRequest>) -> HttpResult {
    // Log the raw JSON for audit purposes
    tracing::info!("Received JSON: {}", json.body());
    
    // Access parsed data directly via Deref
    let user_name = &json.name;
    
    // Forward raw JSON to audit service
    audit_service::log_request(json.body().clone()).await;
    
    // Create response with user data
    format!("Created user: {}", user_name).respond()
}
```

### `ByteBody` - Raw Binary Data Extractor

An extractor for handling raw binary data, perfect for file uploads, image processing, or any binary content.

#### Methods

- `bytes() -> &Vec<u8>` - Get reference to byte buffer
- `into_bytes(self) -> Vec<u8>` - Consume and get byte buffer
- `len() -> usize` - Get buffer length
- `is_empty() -> bool` - Check if buffer is empty
- `as_utf8() -> Result<String, ByteExtractionError>` - Try to convert to UTF-8 string

#### Example

```rust
use foxtive_axum::http::extractors::ByteBody;
use foxtive_axum::http::ext::StructResponseExt;
use foxtive_axum::error::HttpError;

async fn upload_file(body: ByteBody) -> HttpResult {
    if body.is_empty() {
        return "No file data received".respond_code(
            ResponseCode::BadRequest,
            "Empty file upload"
        );
    }
    
    let file_size = body.len();
    
    // Check if it's a valid image by examining magic bytes
    let bytes = body.bytes();
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        // JPEG image
        save_image("jpeg", body.into_bytes()).await?;
        format!("JPEG image uploaded successfully ({} bytes)", file_size).respond()
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        // PNG image
        save_image("png", body.into_bytes()).await?;
        format!("PNG image uploaded successfully ({} bytes)", file_size).respond()
    } else {
        "Unsupported image format".respond_code(
            ResponseCode::BadRequest,
            "Only JPEG and PNG images are supported"
        )
    }
}

async fn save_image(format: &str, data: Vec<u8>) -> Result<(), HttpError> {
    // Implementation for saving image
    Ok(())
}
```

### `StringBody` - UTF-8 String Extractor with Parsing

An extractor that reads the request body as a UTF-8 string with additional parsing utilities.

#### Methods

- `body() -> &String` - Get reference to string body
- `into_body(self) -> String` - Consume and get string body
- `len() -> usize` - Get string length in bytes
- `is_empty() -> bool` - Check if string is empty
- `parse<T: FromStr>() -> Result<T, StringExtractionError>` - Parse string to any type implementing `FromStr`

#### Example

```rust
use foxtive_axum::http::extractors::StringBody;
use foxtive_axum::http::ext::StructResponseExt;

async fn submit_calculation(body: StringBody) -> HttpResult {
    if body.is_empty() {
        return "No calculation provided".respond_code(
            ResponseCode::BadRequest,
            "Request body cannot be empty"
        );
    }
    
    // Parse the string as a number
    match body.parse::<f64>() {
        Ok(number) => {
            let result = number * number;
            format!("{}² = {}", number, result).respond_msg("Calculation completed")
        }
        Err(_) => {
            format!("'{}' is not a valid number", body.body()).respond_code(
                ResponseCode::BadRequest,
                "Invalid number format"
            )
        }
    }
}

async fn parse_config(body: StringBody) -> HttpResult {
    let config_str = body.body();
    
    // Try parsing as JSON first, then fall back to other formats
    if config_str.trim_start().starts_with('{') {
        match serde_json::from_str::<serde_json::Value>(config_str) {
            Ok(json_val) => {
                format!("Parsed JSON config with {} keys", 
                    json_val.as_object().map(|o| o.len()).unwrap_or(0)
                ).respond()
            }
            Err(_) => {
                "Invalid JSON format".respond_code(
                    ResponseCode::BadRequest,
                    "Configuration must be valid JSON"
                )
            }
        }
    } else {
        format!("Raw config: {} characters", config_str.len()).respond()
    }
}
```

### Extractor Error Handling

All custom extractors provide proper error handling with appropriate HTTP status codes:

- **400 Bad Request** - Invalid data format, JSON parsing errors, invalid UTF-8
- **413 Payload Too Large** - Request body exceeds size limits
- **500 Internal Server Error** - Unexpected errors during processing

```rust
use foxtive_axum::http::extractors::JsonBody;

async fn handler(json: JsonBody<MyType>) -> Result<HttpResult, HttpError> {
    // Extractors automatically handle common errors
    // Custom processing errors can be handled explicitly
    
    match validate_business_logic(&json) {
        Ok(_) => Ok("Success".respond()),
        Err(e) => Err(HttpError::AppMessage(e.into())),
    }
}
```

### Creating Responses

Foxtive Axum provides a standardized JSON response format:

```rust
use foxtive_axum::http::ext::StructResponseExt;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn get_user() -> impl IntoResponse {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
    };
    
    // Automatically creates a standardized JSON response
    user.into_response()
}

async fn create_user() -> Result<impl IntoResponse, HttpError> {
    let user = User {
        id: 2,
        name: "Jane Doe".to_string(),
    };
    
    // Respond with a custom message and code
    user.respond_msg("User created successfully")
}
```

### Working with Response Codes

The library provides standard response codes:

```rust
use foxtive_axum::enums::response_code::ResponseCode;
use foxtive_axum::http::ext::StructResponseExt;

async fn not_found() -> HttpResult {
    // Return a 404 Not Found response
    ().respond_code(ResponseCode::NotFound, "Resource not found")
}
```

### Error Handling

Foxtive Axum provides integrated error handling:

```rust
use foxtive_axum::error::HttpError;
use foxtive::Error as FoxtiveError;

async fn fallible_handler() -> Result<impl IntoResponse, HttpError> {
    // Convert application errors to HTTP errors
    let result: Result<(), FoxtiveError> = some_operation().map_err(HttpError::from)?;
    
    // Or return custom errors
    if something_bad_happens() {
        return Err(HttpError::AppMessage(
            "Something went wrong".into()
        ));
    }
    
    "Success".respond()
}
```

### Validation (with `validator` feature)

When the `validator` feature is enabled, you can validate request payloads:

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
    
    #[validate(email)]
    email: String,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Result<impl IntoResponse, HttpError> {
    // Validation happens automatically when using the validator feature
    payload.validate()?;
    
    // Process the valid payload
    // ...
    
    "User created".respond()
}
```

### CORS Configuration

With the `cors` feature enabled:

```rust
use foxtive_axum::server::Server;
use foxtive_axum::setup::FoxtiveAxumSetup;
use axum::http::HeaderValue;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::new(foxtive_setup)
        .allow_origin(HeaderValue::from_static("https://example.com"))
        .allow_method(axum::http::Method::GET)
        .allow_method(axum::http::Method::POST)
        .run()
        .await
}
```

### Static File Serving (with `static` feature)

With the `static` feature enabled:

```rust
use foxtive_axum::server::{Server, StaticFileConfig};

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::new(foxtive_setup)
        .static_config(StaticFileConfig {
            path: "/static".to_string(),
            dir: "./public".to_string(),
        })
        .run()
        .await
}
```

## Advanced Extractor Usage

### Combining Multiple Data Sources

```rust
use foxtive_axum::http::extractors::{JsonBody, StringBody};
use axum::extract::Path;

// Use extractors with other Axum extractors
async fn update_user_with_notes(
    Path(user_id): Path<u64>,
    json: JsonBody<UpdateUserRequest>,
    notes: StringBody,
) -> HttpResult {
    // Log the original JSON for auditing
    audit_log::record_update(user_id, json.body()).await;
    
    // Process the user update
    let user_data = json.inner();
    let additional_notes = notes.body();
    
    // Update user with both structured data and notes
    update_user(user_id, user_data, additional_notes).await?;
    
    "User updated successfully".respond()
}
```

### File Upload with Metadata

```rust
use foxtive_axum::http::extractors::ByteBody;

async fn upload_with_metadata(byte: ByteBody) -> HttpResult {
    let byte = byte.inner();
    
    "Bytes collected successfully".respond()
}
```

## Response Format

All responses follow a standardized JSON format:

```json
{
  "code": "000",
  "success": true,
  "timestamp": 1640995200,
  "message": "Optional message",
  "data": {}
}
```

Where:
- `code`: Application-specific response code (e.g., "000" for success)
- `success`: Boolean indicating success or failure
- `timestamp`: Unix timestamp of the response
- `message`: Optional message providing additional context
- `data`: The actual response data

## Response Codes

| Code | Enum                              | HTTP Status |
|------|-----------------------------------|-------------|
| 000  | ResponseCode::Ok                  | 200         |
| 001  | ResponseCode::Created             | 201         |
| 002  | ResponseCode::Accepted            | 202         |
| 003  | ResponseCode::NoContent           | 204         |
| 004  | ResponseCode::BadRequest          | 400         |
| 005  | ResponseCode::Unauthorized        | 401         |
| 007  | ResponseCode::Forbidden           | 403         |
| 008  | ResponseCode::NotFound            | 404         |
| 009  | ResponseCode::Conflict            | 409         |
| 010  | ResponseCode::InternalServerError | 500         |
| 011  | ResponseCode::ServiceUnavailable  | 503         |
| 012  | ResponseCode::NotImplemented      | 501         |
| 013  | ResponseCode::MethodNotAllowed    | 405         |

## Best Practices

### Choosing the Right Extractor

- **`JsonBody<T>`** - Use when you need both JSON parsing and access to the raw JSON string (logging, forwarding, validation)
- **`ByteBody`** - Use for binary data like file uploads, images, or when you need to inspect raw bytes
- **`StringBody`** - Use for text data that might need parsing (form data, configuration files, simple text processing)

### Performance Considerations

- All extractors read the entire request body into memory
- `JsonBody` performs JSON deserialization once and caches the parsed data

### Error Handling Best Practices

```rust
use foxtive_axum::http::extractors::JsonBody;
use foxtive_axum::enums::response_code::ResponseCode;

async fn robust_handler(json: JsonBody<MyData>) -> HttpResult {
    // Extractors handle basic errors automatically
    // Focus on business logic errors
    
    match process_data(json.inner()) {
        Ok(result) => result.respond_msg("Processing completed"),
        Err(ProcessingError::InvalidData(msg)) => {
            msg.respond_code(ResponseCode::BadRequest, "Invalid input data")
        }
        Err(ProcessingError::ServiceUnavailable) => {
            "Service temporarily unavailable".respond_code(
                ResponseCode::ServiceUnavailable,
                "Please try again later"
            )
        }
        Err(_) => {
            "Internal error occurred".respond_code(
                ResponseCode::InternalServerError,
                "Contact support if this persists"
            )
        }
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT