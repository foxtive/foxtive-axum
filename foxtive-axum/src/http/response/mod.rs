pub mod anyhow;
pub mod ext;
mod message;
pub mod respond;
pub mod result;
pub mod r#struct;
#[cfg(feature = "templating")]
mod view;

#[cfg(feature = "templating")]
pub use view::View;

#[cfg(feature = "templating")]
pub use foxtive::TemplateContext as ViewContext;
