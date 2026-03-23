use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ViewContext;
use axum::http::StatusCode;
use foxtive::FOXTIVE;
use foxtive::prelude::AppStateExt;

pub struct View;

impl View {
    pub fn render(view: &str, ctx: &ViewContext) -> HttpResult {
        let html = FOXTIVE.app().render(view.to_string(), ctx)?;
        Ok(Responder::html(&html, StatusCode::OK))
    }
}
