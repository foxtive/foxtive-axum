use axum::extract::Request;
use axum::Extension;
use foxtive::prelude::{AppMessage, AppResult};
use foxtive::App;
use std::sync::Arc;

/// Extension trait for resolving DI services from the request.
///
/// Provides convenient access to services registered in the [`App`] container.
/// Requires that `Arc<App>` was injected as a request extension (done automatically
/// by the HTTP kernel).
pub trait RequestExt {
    /// Get an owned `Arc<App>` from the request extensions.
    fn app(&self) -> AppResult<Arc<App>>;

    /// Resolve a service from the DI container.
    ///
    /// Shorthand for `req.app()?.require::<T>()`.
    ///
    /// # Errors
    /// Returns an error if `Arc<App>` is not registered or the service type
    /// `T` has not been added to the container.
    fn service<T: Send + Sync + 'static>(&self) -> AppResult<Arc<T>>;

    /// Resolve a service from the DI container, returning `None` if not registered.
    ///
    /// Shorthand for `req.app()?.get::<T>()`.
    fn service_opt<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>;

    /// Resolve a service and clone the inner value.
    ///
    /// Shorthand for `req.app()?.require_cloned::<T>()`.
    ///
    /// # Errors
    /// Returns an error if `Arc<App>` is not registered or the service type
    /// `T` has not been added to the container.
    fn service_cloned<T: Clone + Send + Sync + 'static>(&self) -> AppResult<T>;
}

impl RequestExt for Request {
    fn app(&self) -> AppResult<Arc<App>> {
        self.extensions()
            .get::<Extension<Arc<App>>>()
            .cloned()
            .map(|ext| ext.0)
            .ok_or_else(|| {
                AppMessage::internal_server_error("Arc<App> not registered as request extension")
            })
    }

    fn service<T: Send + Sync + 'static>(&self) -> AppResult<Arc<T>> {
        self.app()?.require::<T>()
    }

    fn service_opt<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.app().ok()?.get::<T>()
    }

    fn service_cloned<T: Clone + Send + Sync + 'static>(&self) -> AppResult<T> {
        self.app()?.require_cloned::<T>()
    }
}
