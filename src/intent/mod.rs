//! Intent parsing — XML to JSON, with shorthand expansion.

pub mod parser;
pub mod types;

pub use parser::parse_xml;
pub use types::ValidatedIntent;
