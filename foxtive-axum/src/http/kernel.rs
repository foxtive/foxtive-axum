use crate::http::HttpResult;
use crate::http::responder::Responder;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Method, Request};
use axum::response::{IntoResponse, Response};
use axum::Extension;
use foxtive::App;
use foxtive::enums::AppMessage;
use std::convert::Infallible;
#[cfg(feature = "static")]
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower::{ServiceBuilder, service_fn};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::enums::response_code::ResponseCode;

/// Kernel configuration holding axum-specific settings for the HTTP kernel.
pub(crate) struct KernelConfig {
    pub(crate) allowed_origins: Vec<HeaderValue>,
    pub(crate) allowed_methods: Vec<Method>,
    pub(crate) allowed_headers: Vec<HeaderName>,
    pub(crate) client_timeout: Duration,
    pub(crate) body_config: crate::server::BodyConfig,
    #[cfg(feature = "rate-limit")]
    pub(crate) rate_limit_config: Option<crate::server::RateLimitConfig>,
    #[cfg(feature = "static")]
    pub(crate) static_file_dir: Option<String>,
    #[cfg(feature = "static")]
    pub(crate) allowed_static_media_extensions: Vec<String>,
}

pub(crate) fn setup(router: Router, app: Arc<App>, config: KernelConfig) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(DefaultOnResponse::new().include_headers(true));

    let cors_layer = if config.allowed_origins.is_empty() {
        // Restrictive default: no CORS headers sent unless explicitly configured
        // This is safer for production - browsers will block cross-origin requests
        CorsLayer::new()
    } else {
        CorsLayer::new()
            .allow_origin(config.allowed_origins.clone())
            .allow_methods(config.allowed_methods.clone())
            .expose_headers(config.allowed_headers.clone())
            .allow_headers(tower_http::cors::Any)
    };

    let builder = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(trace_layer)
        .layer(cors_layer)
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            config.client_timeout,
        ));

    // Apply the main middleware builder layers
    #[allow(unused_mut)]
    let mut router = router.layer(builder);

    // Add rate limiting as a separate layer.
    // GovernorLayer produces BoxError, but axum's Router requires Into<Infallible>,
    // so we wrap it with RateLimitService that converts errors to 429 responses.
    #[cfg(feature = "rate-limit")]
    let router = {
        if let Some(rate_limit) = config.rate_limit_config {
            use tower_governor::governor::GovernorConfigBuilder;
            use tower_governor::GovernorLayer;

            let mut config_builder = GovernorConfigBuilder::default();
            config_builder
                .per_second(rate_limit.period_seconds as u64)
                .burst_size(rate_limit.burst_size);

            let governor_config = Arc::new(config_builder.finish().unwrap());

            let governor_layer = GovernorLayer::new(governor_config);

            router = router.layer(RateLimitLayer { inner: governor_layer });
        }
        router
    };

    // Capture config for fallback handlers
    let fallback_config = Arc::new(FallbackConfig {
        #[cfg(feature = "static")]
        static_file_dir: config.static_file_dir,
        #[cfg(feature = "static")]
        allowed_static_media_extensions: config.allowed_static_media_extensions,
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
        allowed_headers: config.allowed_headers,
    });

    let fallback_config_405 = fallback_config.clone();

    // Inject Arc<App> as a request extension so handlers can extract it
    // via `Extension(app): Extension<Arc<App>>` without needing router state.
    let app_layer = Extension(app);

    // Inject BodyConfig so custom extractors (JsonBody, StringBody, ByteBody)
    // can read configured size limits from request extensions.
    let body_config_layer = Extension(config.body_config);

    router
        .layer(body_config_layer)
        .layer(app_layer)
        .method_not_allowed_fallback(move |req| fallback_405(req, fallback_config_405.clone()))
        .fallback_service(service_fn(move |req| {
            let config = fallback_config.clone();
            async move { fallback_404(req, config).await }
        }))
}

/// Configuration for fallback handlers (captured in closures).
struct FallbackConfig {
    #[cfg(feature = "static")]
    static_file_dir: Option<String>,
    #[cfg(feature = "static")]
    allowed_static_media_extensions: Vec<String>,
    allowed_origins: Vec<HeaderValue>,
    allowed_methods: Vec<Method>,
    allowed_headers: Vec<HeaderName>,
}

#[allow(unused_variables)]
async fn fallback_404(req: Request<Body>, config: Arc<FallbackConfig>) -> Result<Response<Body>, Infallible> {
    #[cfg(feature = "static")]
    {
        use crate::http::static_file::{is_url_a_file, resolve_static_file_path};

        let uri = req.uri().path();

        // check if a static file can be served on this url
        // this is useful to handle static file request at root path
        if let Some(static_file_dir) = &config.static_file_dir
            && is_url_a_file(uri, &config.allowed_static_media_extensions)
        {
            let path = resolve_static_file_path(Path::new(static_file_dir), Path::new(uri));
            if let Ok(contents) = tokio::fs::read(path).await {
                // guess file mime
                let guess = mime_guess::from_path(uri);
                let mut builder = Response::builder().status(axum::http::StatusCode::OK);

                if let Some(mime) = guess.first() {
                    builder = builder.header("Content-Type", mime.as_ref());
                }

                return match builder.body(Body::from(contents.to_vec())) {
                    Ok(response) => Ok(response),
                    Err(err) => {
                        tracing::error!("Error building response: {:?}", err);
                        Ok(Responder::internal_server_error())
                    }
                };
            }
        }
    }

    Ok(Responder::not_found_message(
        "Requested Resource(s) Not Found",
    ))
}

async fn fallback_405(req: Request<Body>, config: Arc<FallbackConfig>) -> HttpResult {
    let origin = req.headers().get("origin");

    if req.method() == Method::OPTIONS {
        let mut response = Response::builder()
            .status(200)
            .body(Body::empty())
            .map_err(|e| AppMessage::Infrastructure {
                message: "Failed to build response".to_string(),
                source: Some(Box::new(e)),
            })?;

        // Add CORS headers manually
        let headers = response.headers_mut();

        // Set Access-Control-Allow-Origin
        if config.allowed_origins.is_empty() {
            // Permissive mode - allow any origin
            if let Some(origin_value) = origin {
                headers.insert("access-control-allow-origin", origin_value.clone());
            } else {
                headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
            }
        } else {
            // Check if the origin is in allowed origins
            if let Some(origin_value) = origin
                && config.allowed_origins.contains(origin_value)
            {
                headers.insert("access-control-allow-origin", origin_value.clone());
            }
        }

        // Set Access-Control-Allow-Methods
        let allowed_methods = config
            .allowed_methods
            .iter()
            .map(|method| method.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let allowed_methods = HeaderValue::from_str(&allowed_methods)
            .map_err(|e| AppMessage::Infrastructure {
                message: "Invalid header value".to_string(),
                source: Some(Box::new(e)),
            })?;

        if !allowed_methods.is_empty() {
            headers.insert("access-control-allow-methods", allowed_methods);
        } else {
            headers.insert(
                "access-control-allow-methods",
                HeaderValue::from_static("GET, POST, PATCH, PUT, DELETE, OPTIONS"),
            );
        }

        // Set Access-Control-Allow-Headers
        let allowed_headers = config.allowed_headers.join(",");

        headers.insert(
            "access-control-allow-headers",
            HeaderValue::from_str(&allowed_headers).unwrap(),
        );

        Ok(response)
    } else {
        Ok(
            Responder::message("Request Method Not Allowed", ResponseCode::MethodNotAllowed)
                .into_response(),
        )
    }
}

/// A layer that wraps GovernorLayer and converts its BoxError into Infallible
/// by turning rate-limit errors into HTTP 429 responses.
#[cfg(feature = "rate-limit")]
struct RateLimitLayer<G: Clone> {
    inner: G,
}

#[cfg(feature = "rate-limit")]
impl<G: Clone> Clone for RateLimitLayer<G> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

#[cfg(feature = "rate-limit")]
impl<G, S> tower::Layer<S> for RateLimitLayer<G>
where
    G: tower::Layer<S> + Clone,
    G::Service: Clone,
{
    type Service = RateLimitService<G::Service>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner: self.inner.layer(inner),
        }
    }
}

/// A service wrapper that converts BoxError to Infallible by producing 429 responses.
#[cfg(feature = "rate-limit")]
struct RateLimitService<S: Clone> {
    inner: S,
}

#[cfg(feature = "rate-limit")]
impl<S: Clone> Clone for RateLimitService<S> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

#[cfg(feature = "rate-limit")]
impl<S, ReqBody> tower::Service<axum::http::Request<ReqBody>> for RateLimitService<S>
where
    S: tower::Service<axum::http::Request<ReqBody>, Response = axum::response::Response>
        + Clone
        + Send
        + 'static,
    S::Error: Into<tower::BoxError> + Send,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = axum::response::Response;
    type Error = Infallible;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        // Governor's poll_ready just delegates to the inner service (axum Router),
        // which has Error = Infallible, so this never actually fails.
        self.inner.poll_ready(cx).map_err(|_| unreachable!("inner service is infallible"))
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        // Swap to maintain readiness invariant
        std::mem::swap(&mut self.inner, &mut inner);

        Box::pin(async move {
            match inner.call(req).await {
                Ok(response) => Ok(response),
                Err(_error) => {
                    let response = axum::response::Response::builder()
                        .status(axum::http::StatusCode::TOO_MANY_REQUESTS)
                        .body(axum::body::Body::from("Too Many Requests"))
                        .unwrap();
                    Ok(response)
                }
            }
        })
    }
}
