use axum::Router;
use axum::routing::post;
use foxtive::Environment;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::trace::Tracing;
use foxtive_axum::enums::response_code::ResponseCode;
use foxtive_axum::error::HttpError;
use foxtive_axum::http::responder::Responder;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::extractors::ByteBody;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create your routes
    let app = Router::new().route("/upload", post(handler));

    // Setup Foxtive core
    let foxtive_setup = FoxtiveSetup {
        env_prefix: "FOXTIVE".to_string(),
        private_key: "".to_string(),
        public_key: "".to_string(),
        app_key: "".to_string(),
        app_code: "BYTE".to_string(),
        app_name: "Byte Extractor".to_string(),
        env: Environment::Local,
        #[cfg(feature = "templating")]
        template_directory: "".to_string(),
    };

    // Configure & run server
    Server::new(foxtive_setup)
        .host("127.0.0.1")
        .port(3000)
        .router(app)
        .tracing(Tracing::minimal())
        .on_started(async { info!("Server started successfully") })
        .run()
        .await
}

async fn handler(body: ByteBody) -> HttpResult {
    if body.is_empty() {
        return "No file data received".respond_code(ResponseCode::BadRequest, "Empty file upload");
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
        Ok(Responder::bad_req_message("Unsupported image format"))
    }
}

async fn save_image(_format: &str, _data: Vec<u8>) -> Result<(), HttpError> {
    Ok(())
}
