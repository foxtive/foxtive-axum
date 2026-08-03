use axum::Router;
use axum::routing::get;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
#[cfg(feature = "rate-limit")]
use foxtive_axum::server::RateLimitConfig;
use foxtive_axum::server::Server;
use std::time::Duration;
use foxtive::{App, Environment};
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container)
    let app = App::builder("Security Example", "SECURITY")
        .environment(Environment::Local)
        .build()
        .await?;

    // Create your routes
    let router = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler));

    // Configure & run server with security features
    let mut server = Server::new(app)
        .host("127.0.0.1")
        .port(3002)
        .router(router)
        .tracing(Tracing::default())
        // Request timeout: 30 seconds (protects against slow clients)
        .client_timeout(Duration::from_secs(30))
        .bootstrap(|app| async move {
            info!("Bootstrapping application: {}", app.app_name());
            info!("Security features enabled:");
            info!("  - Request timeout: 30 seconds");
            #[cfg(feature = "rate-limit")]
            info!("  - Rate limiting: 100 requests/minute");
            info!("  - CORS: Restrictive (no cross-origin by default)");
            Ok(())
        })
        .on_started(async { 
            info!("Server started at http://127.0.0.1:3002");
            info!("Try these endpoints:");
            info!("  - GET /");
            info!("  - GET /health");
        });

    // Add rate limiting if feature is enabled
    #[cfg(feature = "rate-limit")]
    {
        server = server.rate_limit(RateLimitConfig::per_minute(100));
        info!("Rate limiting configured: 100 requests per minute");
    }

    server.run().await
}

async fn root_handler() -> HttpResult {
    "Welcome! This server has security features enabled.".respond()
}

async fn health_handler() -> HttpResult {
    serde_json::json!({
        "status": "healthy",
        "features": {
            "timeout": "30s",
            "rate_limit": if cfg!(feature = "rate-limit") { "100 req/min" } else { "disabled" },
            "cors": "restrictive"
        }
    }).respond()
}
