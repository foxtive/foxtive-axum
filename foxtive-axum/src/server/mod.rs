mod config;

#[cfg(feature = "rate-limit")]
pub use config::RateLimitConfig;
#[cfg(feature = "static")]
pub use config::StaticFileConfig;
pub use config::{BodyConfig, Server};
use std::net::SocketAddr;

use crate::http::kernel;
use crate::server::config::ShutdownSignalHandler;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use tokio::signal;
use tracing::{info, warn};

/// Initialize tracing configuration.
///
/// # Deprecated
/// This function is deprecated. Use `AppBuilder::tracing()` instead for builder-level
/// initialization, which provides better fail-fast behavior and earlier tracing availability.
///
/// # Example (New Pattern)
/// ```rust,no_run
/// use foxtive::App;
/// use foxtive::setup::trace::Tracing;
///
/// # async fn example() -> foxtive::results::AppResult<()> {
/// let app = App::builder("MyApp", "MYAPP")
///     .tracing(Tracing::default())
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[deprecated(
    since = "1.3.0",
    note = "Use AppBuilder::tracing() instead for builder-level initialization"
)]
pub(crate) fn init_bootstrap(_service: &str, config: Tracing) -> AppResult<()> {
    foxtive::setup::trace::init_tracing(config)?;
    Ok(())
}

pub(crate) async fn run(config: Server) -> AppResult<()> {
    if !config.has_started_bootstrap {
        let t_config = config.tracing_config.unwrap_or_default();
        #[allow(deprecated)]
        init_bootstrap(&config.service_name, t_config)?;
    }

    // Install panic hook if enabled
    if config.panic_hook {
        foxtive::setup::panic::install(config.app.env());
    }

    #[allow(unused_mut)]
    let mut app = config.router;

    // Mount nested services (e.g. socket.io) before kernel setup.
    // route_service matches exact paths, so we also register a trailing-slash variant.
    for (path, service) in config.nested_services {
        let slash_path = if path.ends_with('/') {
            path.clone()
        } else {
            format!("{path}/")
        };
        if slash_path != path {
            let cloned = service.clone_box();
            app = cloned.apply_to_router(app, &slash_path);
        }
        app = service.apply_to_router(app, &path);
    }

    #[cfg(feature = "static")]
    let mut static_file_dir: Option<String> = None;

    #[cfg(feature = "static")]
    if cfg!(feature = "static") {
        static_file_dir = Some(config.static_config.dir.clone());
        let dir = tower_http::services::ServeDir::new(&config.static_config.dir);
        app = app.nest_service(&config.static_config.path, dir);
    }

    let allowed_origins = config.allowed_origins.clone();
    let allowed_methods = config.allowed_methods.clone();
    let allowed_headers = config.allowed_headers.clone();

    #[cfg(feature = "static")]
    let allowed_static_media_extensions = config.allowed_static_media_extensions.clone().unwrap_or(
        crate::http::static_file::DEFAULT_STATIC_MEDIA_EXTENSIONS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let foxtive_app = config.app.clone();

    if let Some(bootstrap) = config.bootstrap {
        bootstrap(config.app.clone()).await?;
    }

    let app = kernel::setup(
        app,
        foxtive_app.clone(),
        kernel::KernelConfig {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            client_timeout: config.client_timeout,
            body_config: config.body_config.unwrap_or_default(),
            #[cfg(feature = "rate-limit")]
            rate_limit_config: config.rate_limit_config,
            #[cfg(feature = "static")]
            static_file_dir,
            #[cfg(feature = "static")]
            allowed_static_media_extensions,
        },
    );

    info!("Starting server at {}:{} ...", config.host, config.port);
    let listener = tokio::net::TcpListener::bind((config.host, config.port))
        .await
        .expect("Couldn't bind to the address");

    if let Some(on_server_started) = config.on_started {
        on_server_started.await;
    }

    // Run startup hooks
    foxtive_app.run_startup_hooks().await?;

    let app_for_shutdown = foxtive_app.clone();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(match config.shutdown_signal {
        None => Box::pin(shutdown_signal(config.on_shutdown, app_for_shutdown)),
        Some(signal) => signal,
    })
    .await
    .map_err(|e| foxtive::prelude::AppMessage::Infrastructure {
        message: "Server error".to_string(),
        source: Some(Box::new(e)),
    })?;

    // Run shutdown hooks after server stops
    foxtive_app.run_shutdown_hooks().await;

    Ok(())
}

async fn shutdown_signal(
    app_signal: Option<ShutdownSignalHandler>,
    _app: std::sync::Arc<foxtive::App>,
) {
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

    warn!("Signal received. Shutting down...");

    // Execute app-level shutdown signal
    if let Some(on_server_shutdown) = app_signal {
        on_server_shutdown.await;
    }
}
