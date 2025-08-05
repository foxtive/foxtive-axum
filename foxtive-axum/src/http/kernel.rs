use crate::FoxtiveAxumState;
use crate::enums::response_code::ResponseCode;
use crate::helpers::responder::Responder;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
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

    let builder = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(trace_layer)
        .layer({
            let mut cors = CorsLayer::permissive();

            if !setup.allowed_methods.is_empty() {
                cors = cors.allow_methods(setup.allowed_methods.clone());
            }

            if !setup.allowed_origins.is_empty() {
                cors = cors.allow_origin(setup.allowed_origins.clone());
            }

            cors
        });

    router
        .layer(builder)
        .fallback_service(service_fn(fallback_404))
        .method_not_allowed_fallback(fallback_405)
}

async fn fallback_404(_req: Request<Body>) -> Result<Response, Infallible> {
    Ok(Responder::not_found_message(
        "Requested Resource(s) Not Found",
    ))
}

async fn fallback_405() -> Result<Response, Infallible> {
    Ok(Responder::message(
        "Request Method Not Allowed",
        ResponseCode::MethodNotAllowed,
    ))
}
