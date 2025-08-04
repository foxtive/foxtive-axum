use crate::FOXTIVE_NTEX;
use crate::setup::state::FoxtiveAxumState;
use foxtive::prelude::AppStateExt;
use foxtive::{FOXTIVE, FoxtiveState};
use std::sync::OnceLock;

pub trait FoxtiveAxumExt {
    fn app(&self) -> &FoxtiveAxumState {
        FOXTIVE_NTEX.get().unwrap()
    }

    fn foxtive(&self) -> &FoxtiveState {
        FOXTIVE.app()
    }
}

impl FoxtiveAxumExt for OnceLock<FoxtiveAxumState> {}
