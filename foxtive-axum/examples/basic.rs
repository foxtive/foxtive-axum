use axum::Extension;
use axum::Router;
use axum::routing::get;
use foxtive::App;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
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
    info!("Handling request, app name: {}", app.app_name());
    // Access services: app.get::<MyService>(), app.db(), app.redis(), etc.
    "Hello, World!".respond()
}
