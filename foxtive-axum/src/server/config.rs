use axum::Router;
use axum::http::{HeaderValue, Method};
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::logger::TracingConfig;
use std::time::Duration;

#[cfg(feature = "static")]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: String,
}

pub struct ServerConfig {
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
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) tracing_config: Option<TracingConfig>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    pub(crate) router: Router,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<HeaderValue>,

    /// list of allowed CORS origins
    pub(crate) allowed_methods: Vec<Method>,

    pub(crate) on_server_started: Option<Box<dyn FnOnce()>>,
}

impl ServerConfig {
    pub fn create(host: &str, port: u16, setup: FoxtiveSetup) -> ServerConfig {
        ServerConfig {
            host: host.to_string(),
            port,
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
            on_server_started: None,
            tracing_config: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(
        host: &str,
        port: u16,
        setup: FoxtiveSetup,
        config: StaticFileConfig,
    ) -> ServerConfig {
        Self::create(host, port, setup).static_config(config)
    }

    pub fn app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
    }

    pub fn router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    pub fn tracing_config(mut self, config: TracingConfig) -> Self {
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

    pub fn on_server_started<TB: FnOnce() + 'static>(mut self, func: TB) -> Self {
        self.on_server_started = Some(Box::new(func));
        self
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
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
