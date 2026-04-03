//! Schema engine — loads intent schemas from YAML, validates JSON payloads,
//! and generates agent-facing templates.

pub mod loader;
pub mod template;
pub mod validator;

pub use loader::{IntentSchema, SchemaRegistry};
pub use template::TemplateInfo;
pub use validator::validate;
