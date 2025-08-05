use axum::Router;
use axum::routing::get;
use foxtive::Environment;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use tracing::info;

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
