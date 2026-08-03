# Foxtive Axum

Foxtive Axum is a Rust web framework built on top of [Axum](https://github.com/tokio-rs/axum) that provides standardized response formats, error handling, custom extractors, dependency injection, and utilities for building REST APIs.

## Features

- **Dependency injection** via `Arc<App>` - no global state
- Standardized JSON response format
- Integrated error handling with HTTP status codes
- **Custom request body extractors** with configurable size limits
- CORS support (restrictive by default)
- Static file serving (optional)
- Request validation (optional)
- Rate limiting (optional)
- Tracing and logging integration
- Panic recovery middleware
- Lifecycle hooks (startup/shutdown)

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
foxtive-axum = { version = "1.0" }
```

### Features

Foxtive Axum comes with optional features:

- `cors` - Enables CORS support
- `static` - Enables static file serving
- `validator` - Enables request validation
- `templating` - Enables server-side template rendering
- `rate-limit` - Enables rate limiting via tower-governor
- `timeout` - Enables request timeout middleware

To enable features, add them to your `Cargo.toml`:

```toml
[dependencies]
foxtive-axum = { version = "1.0", features = ["cors", "static", "rate-limit"] }
```

## Usage

### Basic Setup

```rust
use axum::routing::get;
use axum::Router;
use axum::Extension;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::App;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::http::HttpResult;
use foxtive_axum::server::Server;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container)
    let app = App::builder("Basic", "BASIC")
        .environment(foxtive::Environment::Local)
        .build()
        .await?;

    // Create your routes
    let router = Router::new().route("/", get(handler));

    // Configure & run server
    Server::new(app)
        .host("127.0.0.1")
        .port(3000)
        .router(router)
        .tracing(Tracing::default())
        .bootstrap(|app| async move {
            info!("Bootstrapping application: {}", app.app_name());
            Ok(())
        })
        .on_started(async { info!("Server started successfully") })
        .run()
        .await
}

async fn handler(Extension(app): Extension<Arc<App>>) -> HttpResult {
    // Access services: app.get::<MyService>(), app.db(), app.redis(), etc.
    info!("Handling request, app name: {}", app.app_name());
    "Hello, World!".respond()
}
```

## Body Size Configuration

Configure size limits for request body extractors:

```rust
use foxtive_axum::server::{Server, BodyConfig};

let body_config = BodyConfig::default()
    .json_limit(1024 * 1024)      // 1 MB for JSON
    .string_limit(512 * 1024)     // 512 KB for strings
    .byte_limit(5 * 1024 * 1024); // 5 MB for bytes

Server::new(app)
    .body_config(body_config)
    .run()
    .await
```

## Custom Request Body Extractors

Foxtive Axum provides three powerful custom extractors for handling request bodies with enhanced capabilities:

### `JsonBody<T>` - Enhanced JSON Extractor

An extractor that deserializes JSON while preserving the original raw JSON string. Perfect for logging, forwarding, validation, or when you need both parsed data and the original JSON.

#### Methods

- `body() -> &str` - Get reference to raw JSON string
- `into_body(self) -> String` - Consume and get raw JSON string
- `inner() -> &T` - Get reference to deserialized object
- `into_inner(self) -> T` - Consume and get deserialized object
- Implements `Deref` and `DerefMut` for direct access to `T`

#### Example

```rust
use axum::{routing::post, Router};
use foxtive_axum::http::extractors::JsonBody;
use foxtive_axum::http::response::ext::StructResponseExt;
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
use foxtive_axum::http::response::ext::StructResponseExt;

async fn upload_file(body: ByteBody) -> HttpResult {
    if body.is_empty() {
        return "No file data received".respond();
    }
    
    let file_size = body.len();
    format!("Received {} bytes", file_size).respond()
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
use foxtive_axum::http::response::ext::StructResponseExt;

async fn submit_calculation(body: StringBody) -> HttpResult {
    match body.parse::<f64>() {
        Ok(number) => {
            let result = number * number;
            format!("{}² = {}", number, result).respond()
        }
        Err(_) => {
            format!("'{}' is not a valid number", body.body()).respond()
        }
    }
}
```

### Extractor Error Handling

All custom extractors provide proper error handling with appropriate HTTP status codes:

- **400 Bad Request** - Invalid data format, JSON parsing errors, invalid UTF-8
- **413 Payload Too Large** - Request body exceeds configured size limits
- **500 Internal Server Error** - Unexpected errors during processing

### Creating Responses

Foxtive Axum provides a standardized JSON response format:

```rust
use foxtive_axum::http::response::ext::StructResponseExt;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn get_user() -> HttpResult {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
    };
    
    // Creates a standardized JSON response (200 OK)
    user.respond()
}
```

### Working with Response Codes

```rust
use foxtive_axum::enums::response_code::ResponseCode;
use foxtive_axum::http::response::ext::StructResponseExt;

async fn not_found() -> HttpResult {
    ().respond_code(ResponseCode::NotFound, "Resource not found")
}
```

### Error Handling

Foxtive Axum provides integrated error handling:

```rust
use foxtive_axum::error::HttpError;
use foxtive_axum::http::HttpResult;

async fn fallible_handler() -> HttpResult {
    // AppMessage errors are automatically converted to HTTP responses
    let result: Result<(), HttpError> = some_operation()?;
    
    "Success".respond()
}
```

### CORS Configuration

```rust
use foxtive_axum::server::Server;
use axum::http::{HeaderValue, Method};

Server::new(app)
    .allowed_origins(vec![HeaderValue::from_static("https://example.com")])
    .allowed_methods(vec![Method::GET, Method::POST])
    .run()
    .await
```

### Static File Serving (with `static` feature)

```rust
use foxtive_axum::server::{Server, StaticFileConfig};

Server::new(app)
    .static_config(StaticFileConfig {
        path: "/static".to_string(),
        dir: "./public".to_string(),
    })
    .run()
    .await
```

### Rate Limiting (with `rate-limit` feature)

```rust
use foxtive_axum::server::{Server, RateLimitConfig};

Server::new(app)
    .rate_limit(RateLimitConfig::per_minute(100))
    .run()
    .await
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
| 006  | ResponseCode::PaymentRequired     | 402         |
| 007  | ResponseCode::Forbidden           | 403         |
| 008  | ResponseCode::NotFound            | 404         |
| 009  | ResponseCode::Conflict            | 409         |
| 010  | ResponseCode::InternalServerError | 500         |
| 011  | ResponseCode::ServiceUnavailable  | 503         |
| 012  | ResponseCode::NotImplemented      | 501         |
| 013  | ResponseCode::MethodNotAllowed    | 405         |

## Migration from 0.x to 1.0

### Server initialization

```rust
// Before (0.x)
let setup = FoxtiveSetup { /* ... */ };
Server::new(setup).run().await

// After (1.0)
let app = App::builder("MyApp", "MYAPP").build().await?;
Server::new(app).run().await
```

### Accessing application state in handlers

```rust
// Before (0.x) - via FoxtiveState
async fn handler(State(state): State<FoxtiveState>) -> HttpResult { }

// After (1.0) - via Extension
async fn handler(Extension(app): Extension<Arc<App>>) -> HttpResult { }
```

### Error handling

```rust
// Before (0.x) - anyhow-based
async fn handler() -> Result<impl IntoResponse, anyhow::Error> { }

// After (1.0) - AppMessage-based
async fn handler() -> HttpResult { }
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
