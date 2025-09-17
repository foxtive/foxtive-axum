use crate::FoxtiveAxumState;
use crate::enums::response_code::ResponseCode;
use crate::http::responder::Responder;
use crate::http::HttpResult;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::response::{IntoResponse, Response};
use foxtive::Error;
use std::convert::Infallible;
use std::sync::Arc;
use tower::{ServiceBuilder, service_fn};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

pub(crate) fn setup(router: Router, setup: Arc<FoxtiveAxumState>) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(DefaultOnResponse::new().include_headers(true));

    let cors_layer = if setup.allowed_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(setup.allowed_origins.clone())
            .allow_methods(setup.allowed_methods.clone())
            .expose_headers(setup.allowed_headers.clone())
            .allow_headers(tower_http::cors::Any)
    };

    let builder = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(trace_layer)
        .layer(cors_layer);

    let setup_clone = setup.clone();
    router
        .layer(builder)
        .method_not_allowed_fallback(move |req| fallback_405(req, setup_clone.clone()))
        .fallback_service(service_fn(fallback_404))
}

async fn fallback_404(_req: Request<Body>) -> Result<Response, Infallible> {
    Ok(Responder::not_found_message(
        "Requested Resource(s) Not Found",
    ))
}

async fn fallback_405(req: Request<Body>, setup: Arc<FoxtiveAxumState>) -> HttpResult {
    let origin = req.headers().get("origin");

    if req.method() == axum::http::Method::OPTIONS {
        let mut response = Response::builder()
            .status(200)
            .body(Body::empty())
            .map_err(Error::from)?;

        // Add CORS headers manually
        let headers = response.headers_mut();

        // Set Access-Control-Allow-Origin
        if setup.allowed_origins.is_empty() {
            // Permissive mode - allow any origin
            if let Some(origin_value) = origin {
                headers.insert("access-control-allow-origin", origin_value.clone());
            } else {
                headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
            }
        } else {
            // Check if the origin is in allowed origins
            if let Some(origin_value) = origin
                && setup.allowed_origins.contains(origin_value)
            {
                headers.insert("access-control-allow-origin", origin_value.clone());
            }
        }

        // Set Access-Control-Allow-Methods
        let allowed_methods = setup
            .allowed_methods
            .iter()
            .map(|method| method.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let allowed_methods = HeaderValue::from_str(&allowed_methods).map_err(Error::from)?;

        if !allowed_methods.is_empty() {
            headers.insert("access-control-allow-methods", allowed_methods);
        } else {
            headers.insert(
                "access-control-allow-methods",
                HeaderValue::from_static("GET, POST, PATCH, PUT, DELETE, OPTIONS"),
            );
        }

        // Set Access-Control-Allow-Headers
        let allowed_headers = setup.allowed_headers.join(",");

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
