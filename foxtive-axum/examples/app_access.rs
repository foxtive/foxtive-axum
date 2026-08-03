use axum::Extension;
use axum::Router;
use axum::routing::get;
use foxtive::{App, Environment};
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Serialize)]
struct AppInfo {
    name: String,
    environment: String,
    message: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container)
    let app = App::builder("App Access Example", "APP_ACCESS")
        .environment(Environment::Local)
        .build()
        .await?;

    // Create your routes
    let router = Router::new()
        .route("/", get(root_handler))
        .route("/app-info", get(app_info_handler))
        .route("/health", get(health_handler));

    // Configure & run server
    Server::new(app)
        .host("127.0.0.1")
        .port(3001)
        .router(router)
        .tracing(Tracing::default())
        .bootstrap(|app| async move {
            info!("Bootstrapping application: {}", app.app_name());
            // Register services here if needed
            Ok(())
        })
        .on_started(async { 
            info!("Server started at http://127.0.0.1:3001");
            info!("Try these endpoints:");
            info!("  - GET /");
            info!("  - GET /app-info");
            info!("  - GET /health");
        })
        .run()
        .await
}

/// Root handler - basic example of accessing App
async fn root_handler(Extension(app): Extension<Arc<App>>) -> HttpResult {
    info!("Root handler called, app: {}", app.app_name());
    
    "Welcome to foxtive-axum! Check /app-info for details.".respond()
}

/// Handler that returns app information as JSON
async fn app_info_handler(Extension(app): Extension<Arc<App>>) -> HttpResult {
    let info = AppInfo {
        name: app.app_name().to_string(),
        environment: format!("{:?}", app.env()),
        message: "App accessed successfully via Extension extractor!".to_string(),
    };
    
    info!("App info requested: {:?}", info);
    info.respond()
}

/// Health check handler
async fn health_handler(Extension(app): Extension<Arc<App>>) -> HttpResult {
    let health_status = serde_json::json!({
        "status": "healthy",
        "app": app.app_name(),
        "environment": format!("{:?}", app.env()),
    });
    
    health_status.respond()
}
