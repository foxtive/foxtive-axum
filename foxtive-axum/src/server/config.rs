//! Server configuration types and lifecycle management.

use crate::server;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Method, Request};
use axum::response::IntoResponse;
use foxtive::App;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use futures::future::BoxFuture;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tower::Service;

/// Type alias for a boxed future that resolves when the server should shut down.
pub type ShutdownSignalHandler = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Type alias for the bootstrap function called before the server starts.
pub type BootstrapFn = Box<dyn FnOnce(Arc<App>) -> BoxFuture<'static, AppResult<()>> + Send>;

/// Configuration for serving static files from a directory.
#[cfg(feature = "static")]
pub struct StaticFileConfig {
    /// The URL path prefix for static files (e.g. "/static").
    pub path: String,
    /// The filesystem directory to serve files from (e.g. "./public").
    pub dir: String,
}

/// Type-erased wrapper for Tower services registered on the router.
pub(crate) trait ErasedRouteService: Send + Sync {
    fn apply_to_router(self: Box<Self>, router: Router, path: &str) -> Router;
    fn clone_box(&self) -> Box<dyn ErasedRouteService>;
}

struct ErasedService<S> {
    service: S,
}

impl<S> ErasedRouteService for ErasedService<S>
where
    S: Service<Request<Body>, Error = Infallible> + Clone + Send + Sync + 'static,
    S::Response: IntoResponse,
    S::Future: Send,
{
    fn apply_to_router(self: Box<Self>, router: Router, path: &str) -> Router {
        router.route_service(path, self.service)
    }

    fn clone_box(&self) -> Box<dyn ErasedRouteService> {
        Box::new(ErasedService {
            service: self.service.clone(),
        })
    }
}

/// Rate limiting configuration.
///
/// # Example
/// ```rust
/// use foxtive_axum::server::RateLimitConfig;
///
/// // Allow 100 requests per minute
/// let config = RateLimitConfig::per_minute(100);
///
/// // Allow 10 requests per second
/// let config = RateLimitConfig::per_second(10);
///
/// // Custom: 5 requests every 2 seconds
/// let config = RateLimitConfig::custom(5, 2);
/// ```
#[cfg(feature = "rate-limit")]
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in a burst
    pub burst_size: u32,
    /// Time period in seconds for rate replenishment
    pub period_seconds: u32,
}

#[cfg(feature = "rate-limit")]
impl RateLimitConfig {
    /// Create a rate limit configuration allowing N requests per second
    pub fn per_second(burst_size: u32) -> Self {
        Self {
            burst_size,
            period_seconds: 1,
        }
    }

    /// Create a rate limit configuration allowing N requests per minute
    pub fn per_minute(burst_size: u32) -> Self {
        Self {
            burst_size,
            period_seconds: 60,
        }
    }

    /// Create a rate limit configuration allowing N requests per hour
    pub fn per_hour(burst_size: u32) -> Self {
        Self {
            burst_size,
            period_seconds: 3600,
        }
    }

    /// Create custom rate limit configuration
    /// 
    /// # Arguments
    /// * `burst_size` - Maximum requests allowed in a burst
    /// * `period_seconds` - Time in seconds for rate replenishment
    pub fn custom(burst_size: u32, period_seconds: u32) -> Self {
        Self {
            burst_size,
            period_seconds,
        }
    }
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
            json_limit: 2 * 1024 * 1024,    // 2 MB
            string_limit: 2 * 1024 * 1024,  // 2 MB
            byte_limit: 10 * 1024 * 1024,   // 10 MB
        }
    }
}

/// The HTTP server configuration.
///
/// Use the builder pattern to configure host, port, routing, middleware,
/// and lifecycle hooks, then call [`Server::run`] to start serving.
///
/// # Example
/// ```rust,no_run
/// use foxtive_axum::server::Server;
/// use foxtive::App;
/// use std::sync::Arc;
///
/// # async fn example() -> foxtive::results::AppResult<()> {
/// let app = App::builder("MyApp", "MYAPP").build().await?;
///
/// Server::new(app)
///     .host("127.0.0.1")
///     .port(3000)
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct Server {
    /// The foxtive application instance (DI container).
    pub(crate) app: Arc<App>,

    /// The axum router containing user-defined routes.
    pub(crate) router: Router,

    /// Optional bootstrap function called before the server starts.
    pub(crate) bootstrap: Option<BootstrapFn>,

    /// Optional future executed after the server has started.
    pub(crate) on_started: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    /// Optional future executed during graceful shutdown.
    pub(crate) on_shutdown: Option<ShutdownSignalHandler>,

    /// Optional custom shutdown signal (overrides default Ctrl+C / SIGTERM).
    pub(crate) shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    /// The host address to bind to (default: "0.0.0.0").
    pub(crate) host: String,
    /// The port to listen on (default: 8023).
    pub(crate) port: u16,
    /// Number of worker threads (default: 2).
    pub(crate) workers: usize,

    /// Maximum concurrent connections per worker (default: 25,000).
    pub(crate) max_connections: usize,

    /// Maximum connection establishment rate (default: 256).
    pub(crate) max_connections_rate: usize,

    /// Client request read timeout (default: 3s).
    pub(crate) client_timeout: Duration,

    /// Client disconnect timeout (default: 5s).
    pub(crate) client_disconnect: Duration,

    /// TCP keep-alive interval (default: 5s).
    pub(crate) keep_alive: Duration,

    /// Maximum pending connection backlog (default: 2048).
    pub(crate) backlog: i32,

    /// Body size limit configuration (applied via Extension layer).
    pub(crate) body_config: Option<BodyConfig>,

    /// Service name used for env loading and tracing.
    pub(crate) service_name: String,

    /// Optional tracing configuration.
    pub(crate) tracing_config: Option<Tracing>,

    #[cfg(feature = "rate-limit")]
    pub(crate) rate_limit_config: Option<RateLimitConfig>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    #[cfg(feature = "templating")]
    pub(crate) template_directory: String,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<HeaderValue>,

    /// list of allowed CORS methods
    pub(crate) allowed_methods: Vec<Method>,

    /// list of allowed CORS headers
    pub(crate) allowed_headers: Vec<HeaderName>,

    /// list of allowed static media extensions
    #[cfg(feature = "static")]
    pub(crate) allowed_static_media_extensions: Option<Vec<String>>,

    /// Services to mount at specific paths (e.g. socket.io).
    /// Registered before kernel setup so they don't hit the fallback.
    pub(crate) nested_services: Vec<(String, Box<dyn ErasedRouteService>)>,
}

impl Server {
    /// Create a new server with the given application instance.
    pub fn new(app: Arc<App>) -> Server {
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
            service_name: "foxtive".to_string(),
            app,
            #[cfg(feature = "rate-limit")]
            rate_limit_config: None,
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
            nested_services: vec![],
        }
    }

    /// Set the rate limiting configuration.
    ///
    /// This protects your API from abuse by limiting the number of requests
    /// a client can make within a time period.
    ///
    /// # Example
    /// ```rust
    /// use foxtive_axum::server::{Server, RateLimitConfig};
    /// use foxtive::App;
    /// use std::sync::Arc;
    ///
    /// # async fn example(app: Arc<App>) {
    /// // Allow 100 requests per minute
    /// let server = Server::new(app)
    ///     .rate_limit(RateLimitConfig::per_minute(100));
    /// # }
    /// ```
    #[cfg(feature = "rate-limit")]
    pub fn rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    /// Set the HTTP body extraction configuration.
    ///
    /// This allows you to configure size limits for JSON, String, and Byte extractors.
    ///
    /// # Example
    /// ```rust
    /// use foxtive_axum::server::{Server, BodyConfig};
    /// use foxtive::App;
    /// use std::sync::Arc;
    ///
    /// # async fn example(app: Arc<App>) {
    /// let config = BodyConfig::default()
    ///     .json_limit(1024 * 1024); // 1 MB
    ///
    /// let server = Server::new(app).body_config(config);
    /// # }
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

    /// Create a server pre-configured with static file serving.
    #[cfg(feature = "static")]
    pub fn create_with_static(app: Arc<App>, config: StaticFileConfig) -> Server {
        Self::new(app).static_config(config)
    }

    /// Set the list of allowed static media file extensions.
    #[cfg(feature = "static")]
    pub fn static_media_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_static_media_extensions = Some(extensions);
        self
    }

    /// Set the server host address.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the service name (used for env loading and tracing).
    pub fn service_name(mut self, name: &str) -> Self {
        self.service_name = name.to_string();
        self
    }

    /// Set the axum router with application routes.
    pub fn router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    /// Set the tracing configuration.
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

    /// Set the list of allowed CORS origins.
    pub fn allowed_origins(mut self, origins: Vec<HeaderValue>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Set the list of allowed CORS methods.
    pub fn allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    /// Set the list of allowed CORS headers.
    pub fn allowed_headers(mut self, headers: Vec<HeaderName>) -> Self {
        self.allowed_headers = headers;
        self
    }

    /// Set the static file serving configuration.
    #[cfg(feature = "static")]
    pub fn static_config(mut self, static_config: StaticFileConfig) -> Self {
        self.static_config = static_config;
        self
    }

    /// Set the directory for server-side templates.
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
    /// Note: If a custom `shutdown_signal` is also provided using [`shutdown_signal`],
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

    /// Provide a function to execute before the server starts.
    ///
    /// The function receives an `Arc<App>` which provides access to all
    /// registered services, configuration, and lifecycle methods.
    pub fn bootstrap<F, Fut>(mut self, func: F) -> Self
    where
        F: FnOnce(Arc<App>) -> Fut + Send + 'static,
        Fut: Future<Output = AppResult<()>> + Send + 'static,
    {
        self.bootstrap = Some(Box::new(|app| Box::pin(func(app))));
        self
    }

    /// Mark whether the bootstrap has already been started externally.
    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
    }

    /// Start the HTTP server and begin accepting connections.
    pub async fn run(self) -> AppResult<()> {
        server::run(self).await
    }

    /// Mount a Tower service at a path (e.g. `/socket.io`).
    ///
    /// Uses `route_service` under the hood, matching exact paths only.
    /// A trailing-slash variant is also registered automatically.
    ///
    /// # Example
    /// ```rust,no_run
    /// use socketioxide::SocketIo;
    /// # use foxtive_axum::server::Server;
    /// # use std::sync::Arc;
    /// # use foxtive::App;
    /// # async fn example(app: Arc<App>) {
    /// let (svc, io) = SocketIo::new_svc();
    /// io.ns("/", |socket: socketioxide::extract::SocketRef| {});
    ///
    /// Server::new(app)
    ///     .nest_service("/socket.io", svc)
    ///     .run()
    ///     .await;
    /// # }
    /// ```
    pub fn nest_service<S>(mut self, path: &str, service: S) -> Self
    where
        S: Service<Request<Body>, Error = Infallible> + Clone + Send + Sync + 'static,
        S::Response: IntoResponse,
        S::Future: Send,
    {
        self.nested_services.push((
            path.to_string(),
            Box::new(ErasedService { service }),
        ));
        self
    }

    /// Init tracing and load env vars.
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
