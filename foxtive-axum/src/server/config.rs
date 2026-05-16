use crate::{FoxtiveAxumState, server};
use axum::Router;
use axum::http::{HeaderName, HeaderValue, Method};
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::trace::Tracing;
use futures::future::BoxFuture;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub type ShutdownSignalHandler = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub type BootstrapFn =
    Box<dyn FnOnce(Arc<FoxtiveAxumState>) -> BoxFuture<'static, AppResult<()>> + Send>;

#[cfg(feature = "static")]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: String,
}

/// Configuration for HTTP request body extraction.
///
/// This struct controls size limits for different body types (JSON, String, and Byte).
///
/// # Default Settings
/// - JSON body limit: 2 MB
/// - String body limit: 2 MB
/// - Byte body limit: 10 MB
///
/// # Example
/// ```rust
/// use foxtive_axum::server::BodyConfig;
///
/// // Use default configuration
/// let config = BodyConfig::default();
///
/// // Custom limits for different body types
/// let config = BodyConfig::default()
///     .json_limit(1024 * 1024)      // 1 MB for JSON
///     .string_limit(512 * 1024)     // 512 KB for strings
///     .byte_limit(5 * 1024 * 1024); // 5 MB for bytes
/// ```
#[derive(Clone, Debug)]
pub struct BodyConfig {
    /// Maximum allowed size for JSON request bodies in bytes
    pub json_limit: usize,

    /// Maximum allowed size for String request bodies in bytes
    pub string_limit: usize,

    /// Maximum allowed size for Byte request bodies in bytes
    pub byte_limit: usize,
}

impl BodyConfig {
    /// Set the JSON body size limit in bytes
    pub fn json_limit(mut self, limit: usize) -> Self {
        self.json_limit = limit;
        self
    }

    /// Set the String body size limit in bytes
    pub fn string_limit(mut self, limit: usize) -> Self {
        self.string_limit = limit;
        self
    }

    /// Set the Byte body size limit in bytes
    pub fn byte_limit(mut self, limit: usize) -> Self {
        self.byte_limit = limit;
        self
    }
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            json_limit: 2 * 1024 * 1024,   // 2 MB
            string_limit: 2 * 1024 * 1024, // 2 MB
            byte_limit: 10 * 1024 * 1024,  // 10 MB
        }
    }
}

pub struct Server {
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) router: Router,

    pub(crate) bootstrap: Option<BootstrapFn>,

    pub(crate) on_started: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    pub(crate) on_shutdown: Option<ShutdownSignalHandler>,

    pub(crate) shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) workers: usize,

    pub(crate) max_connections: usize,

    pub(crate) max_connections_rate: usize,

    pub(crate) client_timeout: Duration,

    pub(crate) client_disconnect: Duration,

    pub(crate) keep_alive: Duration,

    pub(crate) backlog: i32,

    pub(crate) body_config: Option<BodyConfig>,

    pub(crate) app: String,

    pub(crate) tracing_config: Option<Tracing>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    #[cfg(feature = "templating")]
    pub(crate) template_directory: String,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<HeaderValue>,

    /// list of allowed CORS origins
    pub(crate) allowed_methods: Vec<Method>,

    /// list of allowed CORS headers
    pub(crate) allowed_headers: Vec<HeaderName>,

    /// list of allowed static media extensions
    #[cfg(feature = "static")]
    pub(crate) allowed_static_media_extensions: Option<Vec<String>>,
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
            body_config: None,
            app: "foxtive".to_string(),
            foxtive_setup: setup,
            #[cfg(feature = "static")]
            static_config: StaticFileConfig::default(),
            #[cfg(feature = "templating")]
            template_directory: "resources/templates".to_string(),
            has_started_bootstrap: false,
            router: Router::new(),
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            tracing_config: None,
            on_started: None,
            on_shutdown: None,
            bootstrap: None,
            #[cfg(feature = "static")]
            allowed_static_media_extensions: None,
            shutdown_signal: None,
        }
    }

    /// Set the HTTP body extraction configuration.
    ///
    /// This allows you to configure size limits for JSON, String, and Byte extractors.
    ///
    /// # Example
    /// ```rust
    /// use foxtive_axum::server::{Server, BodyConfig};
    /// use foxtive::setup::FoxtiveSetup;
    /// use foxtive::Environment;
    ///
    /// let setup = FoxtiveSetup {
    ///     env_prefix: "APP".to_string(),
    ///     private_key: "".to_string(),
    ///     public_key: "".to_string(),
    ///     app_key: "".to_string(),
    ///     app_code: "TEST".to_string(),
    ///     app_name: "Test".to_string(),
    ///     env: Environment::Local,
    ///     #[cfg(feature = "templating")]
    ///     template_directory: "".to_string(),
    /// };
    ///
    /// let config = BodyConfig::default()
    ///     .json_limit(1024 * 1024); // 1 MB
    ///
    /// let server = Server::new(setup).body_config(config);
    /// ```
    pub fn body_config(mut self, body_config: BodyConfig) -> Self {
        self.body_config = Some(body_config);
        self
    }

    /// Deprecated: Use body_config() instead
    #[deprecated(since = "0.13.0", note = "Use body_config() instead")]
    pub fn json_config(mut self, limit: usize) -> Self {
        let mut body_config = self.body_config.unwrap_or_default();
        body_config.json_limit = limit;
        self.body_config = Some(body_config);
        self
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(setup: FoxtiveSetup, config: StaticFileConfig) -> Server {
        Self::new(setup).static_config(config)
    }

    #[cfg(feature = "static")]
    pub fn static_media_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_static_media_extensions = Some(extensions);
        self
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

    pub fn allowed_origins(mut self, origins: Vec<HeaderValue>) -> Self {
        self.allowed_origins = origins;
        self
    }

    pub fn allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    pub fn allowed_headers(mut self, headers: Vec<HeaderName>) -> Self {
        self.allowed_headers = headers;
        self
    }

    #[cfg(feature = "static")]
    pub fn static_config(mut self, static_config: StaticFileConfig) -> Self {
        self.static_config = static_config;
        self
    }

    #[cfg(feature = "templating")]
    pub fn template_directory<D: AsRef<std::ffi::OsStr> + ?Sized>(mut self, dir: &D) -> Self {
        self.template_directory = dir.as_ref().to_os_string().into_string().unwrap();
        self
    }

    /// Provide a function to execute after the server starts
    pub fn on_started<F>(mut self, handler: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.on_started = Some(Box::pin(handler));
        self
    }

    /// Sets a custom shutdown handler to be called when the application is shutting down.
    ///
    /// This method allows you to provide a future that will be awaited during shutdown.
    /// It is typically used to perform cleanup tasks like closing database connections,
    /// flushing logs, or other async teardown operations.
    ///
    /// **Note:** If a custom `shutdown_signal` is also provided using [`shutdown_signal`],
    /// that will take precedence over this handler, and this `on_shutdown` handler will
    /// **not** be executed.
    ///
    pub fn on_shutdown<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.on_shutdown = Some(Box::pin(func));
        self
    }

    /// Sets a custom shutdown signal handler that determines when the application should begin shutting down.
    ///
    /// This method allows you to provide a future that, when resolved, triggers the application shutdown.
    /// It is typically used to listen for signals like `Ctrl+C` or system termination requests (`SIGTERM`).
    ///
    /// If this shutdown signal is provided, it will override any handler set using [`on_shutdown`].
    pub fn shutdown_signal<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.shutdown_signal = Some(Box::pin(func));
        self
    }

    /// Provide a function to execute before the server starts
    pub fn bootstrap<F, Fut>(mut self, func: F) -> Self
    where
        F: FnOnce(Arc<FoxtiveAxumState>) -> Fut + Send + 'static,
        Fut: Future<Output = AppResult<()>> + Send + 'static,
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
