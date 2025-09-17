use crate::enums::response_code::ResponseCode;
use crate::http::responder::Responder;
use crate::http::HttpResult;
use crate::FoxtiveAxumState;
use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::response::{IntoResponse, Response};
use axum::Router;
use foxtive::Error;
use std::convert::Infallible;
use std::sync::Arc;
use tower::{service_fn, ServiceBuilder};
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

#[allow(unused_variables)]
async fn fallback_404(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    #[cfg(feature = "static")]
    {
        use crate::http::static_file::{is_url_a_file, resolve_static_file_path};
        use crate::{FoxtiveAxumExt, FOXTIVE_AXUM};

        let uri = req.uri().path();
        let static_file_dir = &FOXTIVE_AXUM.app().static_file_dir;

        // check if a static file can be served on this url
        // this is useful to handle static file request at root path
        if let Some(static_file_dir) = static_file_dir
            && is_url_a_file(uri)
        {
            let path = resolve_static_file_path(static_file_dir.as_ref(), uri.as_ref());
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
