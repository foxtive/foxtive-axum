use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ViewContext;
use axum::http::StatusCode;
use foxtive::App;

/// Template view renderer.
///
/// Renders server-side templates using the foxtive templating engine
/// and returns them as HTML responses.
pub struct View;

impl View {
    /// Render a template with the given context and return an HTML response.
    pub fn render(app: &App, view: &str, ctx: &ViewContext) -> HttpResult {
        let html = app.render(view.to_string(), ctx)?;
        Ok(Responder::html(&html, StatusCode::OK))
    }
}
