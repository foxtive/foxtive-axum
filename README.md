# Foxtive Axum

Foxtive Axum is a Rust web framework built on top of [Axum](https://github.com/tokio-rs/axum) that provides standardized response formats, error handling, and utilities for building REST APIs.

## Features

- Standardized JSON response format
- Integrated error handling with HTTP status codes
- CORS support
- Static file serving (optional)
- Request validation (optional)
- Tracing and logging integration
- Panic recovery middleware

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
foxtive-axum = { version = "0.1.0" }
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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT