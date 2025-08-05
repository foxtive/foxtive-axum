use crate::{server, FoxtiveAxumState};
use axum::http::{HeaderValue, Method};
use axum::Router;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::setup::FoxtiveSetup;
use futures::future::BoxFuture;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

pub type BootstrapFn =
    Box<dyn FnOnce(Arc<FoxtiveAxumState>) -> BoxFuture<'static, AppResult<()>> + Send + Sync>;

#[cfg(feature = "static")]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: String,
}

pub struct Server {
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) router: Router,

    pub(crate) bootstrap: Option<BootstrapFn>,

    pub(crate) on_started: Option<Box<dyn FnOnce()>>,

    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) workers: usize,

    pub(crate) max_connections: usize,

    pub(crate) max_connections_rate: usize,

    pub(crate) client_timeout: Duration,

    pub(crate) client_disconnect: Duration,

    pub(crate) keep_alive: Duration,

    pub(crate) backlog: i32,

    pub(crate) app: String,

    pub(crate) tracing_config: Option<Tracing>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<HeaderValue>,

    /// list of allowed CORS origins
    pub(crate) allowed_methods: Vec<Method>,
}

impl Server {
    pub fn new(setup: FoxtiveSetup) -> Server {
        Server {
            port: 8023,
            host: "0.0.0.0".to_string(),
            workers: 2,
            max_connections: 25_000,
            max_connections_rate: 256,
            client_timeout: Duration::from_secs(3),
            client_disconnect: Duration::from_secs(5),
            keep_alive: Duration::from_secs(5),
            backlog: 2048,
            app: "foxtive".to_string(),
            foxtive_setup: setup,
            #[cfg(feature = "static")]
            static_config: StaticFileConfig::default(),
            has_started_bootstrap: false,
            router: Router::new(),
            allowed_origins: vec![],
            allowed_methods: vec![],
            tracing_config: None,
            on_started: None,
            bootstrap: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(setup: FoxtiveSetup, config: StaticFileConfig) -> Server {
        Self::new(setup).static_config(config)
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
    }

    pub fn router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    pub fn tracing(mut self, config: Tracing) -> Self {
        self.tracing_config = Some(config);
        self
    }

    /// Set number of workers to start.
    ///
    /// By default http server uses 2
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Set the maximum number of pending connections.
    ///
    /// This refers to the number of clients that can be waiting to be served.
    /// Exceeding this number results in the client getting an error when
    /// attempting to connect. It should only affect servers under significant
    /// load.
    ///
    /// Generally set in the 64-2048 range. Default value is 2048.
    ///
    /// This method should be called before `bind()` method call.
    pub fn backlog(mut self, backlog: i32) -> Self {
        self.backlog = backlog;
        self
    }

    /// Set server keep-alive setting.
    ///
    /// By default keep alive is set to a 5 seconds.
    pub fn keep_alive(mut self, d: Duration) -> Self {
        self.keep_alive = d;
        self
    }

    /// Set request read timeout in seconds.
    ///
    /// Defines a timeout for reading client request headers. If a client does not transmit
    /// the entire set headers within this time, the request is terminated with
    /// the 408 (Request Time-out) error.
    ///
    /// To disable timeout set value to 0.
    ///
    /// By default client timeout is set to 3 seconds.
    pub fn client_timeout(mut self, d: Duration) -> Self {
        self.client_timeout = d;
        self
    }

    /// Set server connection disconnect timeout in seconds.
    ///
    /// Defines a timeout for shutdown connection. If a shutdown procedure does not complete
    /// within this time, the request is dropped.
    ///
    /// To disable timeout set value to 0.
    ///
    /// By default client timeout is set to 5 seconds.
    pub fn client_disconnect(mut self, d: Duration) -> Self {
        self.client_disconnect = d;
        self
    }

    /// Sets the maximum per-worker number of concurrent connections.
    ///
    /// All socket listeners will stop accepting connections when this limit is reached
    /// for each worker.
    ///
    /// By default max connections is set to a 25k.
    pub fn max_conn(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Sets the maximum per-worker concurrent connection establish process.
    ///
    /// All listeners will stop accepting connections when this limit is reached. It
    /// can be used to limit the global SSL CPU usage.
    ///
    /// By default max connections is set to a 256.
    pub fn max_conn_rate(mut self, max: usize) -> Self {
        self.max_connections_rate = max;
        self
    }

    pub fn allowed_origins(mut self, allowed_origins: Vec<HeaderValue>) -> Self {
        self.allowed_origins = allowed_origins;
        self
    }

    pub fn allowed_methods(mut self, allowed_methods: Vec<Method>) -> Self {
        self.allowed_methods = allowed_methods;
        self
    }

    #[cfg(feature = "static")]
    pub fn static_config(mut self, static_config: StaticFileConfig) -> Self {
        self.static_config = static_config;
        self
    }

    /// Provide a function to execute after the server starts
    pub fn on_started<TB: FnOnce() + 'static>(mut self, func: TB) -> Self {
        self.on_started = Some(Box::new(func));
        self
    }

    /// Provide a function to execute before the server starts
    pub fn bootstrap<F, Fut>(mut self, func: F) -> Self
    where
        F: FnOnce(Arc<FoxtiveAxumState>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = AppResult<()>> + Send + Sync + 'static,
    {
        self.bootstrap = Some(Box::new(|state| Box::pin(func(state))));
        self
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
    }

    pub async fn run(self) -> AppResult<()> {
        server::run(self).await
    }

    /// Init tracing and load environment variables
    pub fn init_bootstrap(service: &str, config: Tracing) -> AppResult<()> {
        server::init_bootstrap(service, config)
    }
}

#[cfg(feature = "static")]
impl Default for StaticFileConfig {
    fn default() -> Self {
        Self {
            path: "static".to_string(),
            dir: "./static".to_string(),
        }
    }
}
