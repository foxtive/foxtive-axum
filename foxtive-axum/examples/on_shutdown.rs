use axum::Router;
use axum::routing::get;
use foxtive::App;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container) with tracing and env configuration
    let app = App::builder("Shutdown Event Handler", "ON_SHUTDOWN")
        .environment(foxtive::Environment::Local)
        .tracing(Tracing::default())
        .build()
        .await?;

    // Create your routes
    let router = Router::new().route("/", get(handler));

    // Configure & run server
    Server::new(app)
        .host("127.0.0.1")
        .port(3000)
        .router(router)
        .has_started_bootstrap(true) // Tracing already initialized above
        .on_started(async { info!("Server started successfully") })
        .on_shutdown(async {
            warn!("Server shutting down ...");
        })
        .run()
        .await
}

async fn handler() -> HttpResult {
    "Hello, World!".respond()
}
