use crate::FOXTIVE_AXUM;
use axum::http::{HeaderName, HeaderValue, Method};
use foxtive::prelude::AppMessage;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use state::FoxtiveAxumState;
use std::sync::Arc;
use tracing::debug;

pub mod state;

pub struct FoxtiveAxumSetup {
    pub allowed_origins: Vec<HeaderValue>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<HeaderName>,
    pub foxtive_setup: FoxtiveSetup,
}

pub(crate) async fn make_state(setup: FoxtiveAxumSetup) -> AppResult<Arc<FoxtiveAxumState>> {
    debug!("Creating Foxtive-Axum state");
    let app = create_state(&setup).await;

    debug!("Creating Foxtive state");
    foxtive::setup::make_state(setup.foxtive_setup).await?;

    FOXTIVE_AXUM.set(app.clone()).map_err(|_| {
        AppMessage::InternalServerErrorMessage("failed to set up foxtive-axum").ae()
    })?;

    Ok(Arc::new(app))
}

async fn create_state(setup: &FoxtiveAxumSetup) -> FoxtiveAxumState {
    FoxtiveAxumState {
        allowed_origins: setup.allowed_origins.clone(),
        allowed_methods: setup.allowed_methods.clone(),
        allowed_headers: setup.allowed_headers.clone(),
    }
}
