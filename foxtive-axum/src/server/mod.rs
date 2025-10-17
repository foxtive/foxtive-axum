mod config;

pub use config::Server;
#[cfg(feature = "static")]
pub use config::StaticFileConfig;
use std::net::SocketAddr;

use crate::http::kernel;
use crate::server::config::ShutdownSignalHandler;
use crate::setup::{make_state, FoxtiveAxumSetup};
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use foxtive::setup::trace::Tracing;
use foxtive::Error;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

pub(crate) fn init_bootstrap(service: &str, config: Tracing) -> AppResult<()> {
    foxtive::setup::trace::init_tracing(config)?;
    load_environment_variables(service);
    Ok(())
}

pub(crate) async fn run(config: Server) -> AppResult<()> {
    if !config.has_started_bootstrap {
        let t_config = config.tracing_config.unwrap_or_default();
        init_bootstrap(&config.app, t_config).expect("failed to init bootstrap: ");
    }

    #[allow(unused_mut)]
    let mut app = config.router.layer(TraceLayer::new_for_http());

    #[allow(unused_mut)]
    let mut static_file_dir: Option<String> = None;

    #[cfg(feature = "static")]
    if cfg!(feature = "static") {
        app = {
            static_file_dir = Some(config.static_config.dir.clone());
            let dir = tower_http::services::ServeDir::new(config.static_config.dir);
            app.nest_service(&config.static_config.path, dir)
        };
    }

    let state = make_state(FoxtiveAxumSetup {
        static_file_dir,
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
        allowed_headers: config.allowed_headers,
        foxtive_setup: config.foxtive_setup,
        #[cfg(feature = "static")]
        allowed_static_media_extensions: config.allowed_static_media_extensions,
    })
    .await?;

    if let Some(bootstrap) = config.bootstrap {
        bootstrap(state.clone()).await?;
    }

    let app = kernel::setup(app, state);

    info!("Starting server at {}:{} ...", config.host, config.port);
    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .expect("Couldn't bind to the address");

    if let Some(on_server_started) = config.on_started {
        on_server_started.await;
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(match config.shutdown_signal {
        None => Box::pin(shutdown_signal(config.on_shutdown)),
        Some(signal) => signal,
    })
    .await
    .map_err(Error::from)
}

async fn shutdown_signal(app_signal: Option<ShutdownSignalHandler>) {
    // Wait for SIGINT (Ctrl+C) or SIGTERM (in k8s or docker)
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // No-op on non-Unix

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Signal received. Shutting down...");

    // Execute app-level shutdown signal
    if let Some(on_server_shutdown) = app_signal {
        on_server_shutdown.await;
    }
}
