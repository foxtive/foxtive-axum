use axum::Router;
use axum::routing::get;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::{App, Environment};
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container)
    let app = App::builder("Shutdown Signal", "SHUTDOWN_SIGNAL")
        .environment(Environment::Local)
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
        .on_started(async { info!("Server started successfully") })
        .shutdown_signal(shutdown_signal())
        .run()
        .await
}

async fn handler() -> HttpResult {
    "Hello, World!".respond()
}

async fn shutdown_signal() {
    // Wait for SIGINT (Ctrl+C) or SIGTERM (in k8s or docker)
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // No-op on non-Unix

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("[Custom] Signal received. Shutting down...");
}
