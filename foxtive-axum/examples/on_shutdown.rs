use axum::Router;
use axum::routing::get;
use foxtive::Environment;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create your routes
    let app = Router::new().route("/", get(handler));

    // Setup Foxtive core
    let foxtive_setup = FoxtiveSetup {
        env_prefix: "FOXTIVE".to_string(),
        private_key: "".to_string(),
        public_key: "".to_string(),
        app_key: "".to_string(),
        app_code: "ON_SHUTDOWN".to_string(),
        app_name: "Shutdown Event Handler".to_string(),
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
        .on_shutdown(async {
            warn!("Server shutting down ...");
        })
        .run()
        .await
}

async fn handler() -> HttpResult {
    "Hello, World!".respond()
}
