mod config;

pub use config::ServerConfig;
#[cfg(feature = "static")]
pub use config::StaticFileConfig;

use crate::FoxtiveAxumState;
use crate::http::kernel;
use crate::setup::{FoxtiveAxumSetup, make_ntex_state};
use foxtive::Error;
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use foxtive::setup::logger::TracingConfig;
use std::future::Future;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

pub fn init_bootstrap(service: &str, config: TracingConfig) -> AppResult<()> {
    foxtive::setup::logger::init_tracing(config)?;
    load_environment_variables(service);
    Ok(())
}

pub async fn start_axum_server<Callback, Fut>(
    config: ServerConfig,
    callback: Callback,
) -> AppResult<()>
where
    Callback: FnOnce(FoxtiveAxumState) -> Fut + Copy + Send + 'static,
    Fut: Future<Output = AppResult<()>> + Send + 'static,
{
    if !config.has_started_bootstrap {
        let t_config = config.tracing_config.unwrap_or_default();
        init_bootstrap(&config.app, t_config).expect("failed to init bootstrap: ");
    }

    let app_state = make_ntex_state(FoxtiveAxumSetup {
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
        foxtive_setup: config.foxtive_setup,
    })
    .await;

    match callback(app_state.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("app bootstrap callback returned error: {err:?}");
            panic!("boostrap failed");
        }
    }

    #[allow(unused_mut)]
    let mut app = config.router.layer(TraceLayer::new_for_http());

    #[cfg(feature = "static")]
    if cfg!(feature = "static") {
        app = {
            let dir = tower_http::services::ServeDir::new(config.static_config.dir);
            app.nest_service(&config.static_config.path, dir)
        };
    }

    let app = kernel::setup(app, &app_state);

    info!("starting server at {}:{} ...", config.host, config.port);
    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .expect("Couldn't bind to the address");

    if let Some(on_server_started) = config.on_server_started {
        on_server_started();
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(Error::from)
}

async fn shutdown_signal() {
    // Wait for SIGINT (Ctrl+C) or SIGTERM (in k8s or docker)
    let ctrl_c = async {
        signal::ctrl_c()
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

    println!("\nSignal received. Shutting down...");
}
