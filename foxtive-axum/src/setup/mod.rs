use crate::FOXTIVE_NTEX;
use axum::http::{HeaderValue, Method};
use foxtive::setup::FoxtiveSetup;
use state::FoxtiveAxumState;
use std::sync::Arc;

pub mod state;

pub struct FoxtiveAxumSetup {
    pub allowed_origins: Vec<HeaderValue>,
    pub allowed_methods: Vec<Method>,
    pub foxtive_setup: FoxtiveSetup,
}

pub(crate) async fn make_state(setup: FoxtiveAxumSetup) -> Arc<FoxtiveAxumState> {
    let app = create_app_state(&setup).await;

    foxtive::setup::make_state(setup.foxtive_setup).await;

    FOXTIVE_NTEX
        .set(app.clone())
        .expect("failed to set up foxtive-ntex");

    Arc::new(app)
}

async fn create_app_state(setup: &FoxtiveAxumSetup) -> FoxtiveAxumState {
    FoxtiveAxumState {
        allowed_origins: setup.allowed_origins.clone(),
        allowed_methods: setup.allowed_methods.clone(),
    }
}
