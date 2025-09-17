use crate::http::responder::Responder;
use crate::http::response::ViewContext;
use crate::http::HttpResult;
use axum::http::StatusCode;
use foxtive::prelude::AppStateExt;
use foxtive::FOXTIVE;

pub struct View;

impl View {
    pub fn render(view: &str, ctx: ViewContext) -> HttpResult {
        let html = FOXTIVE.app().render(view.to_string(), ctx)?;
        Ok(Responder::html(&html, StatusCode::OK))
    }
}
