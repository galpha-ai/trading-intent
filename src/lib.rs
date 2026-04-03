//! TIM — Trading Intent Model
//!
//! Schema-driven intent gateway: parse XML, validate against configurable
//! intent schemas, serve templates for AI agents, dispatch to executors.

pub mod config;
pub mod dispatch;
pub mod http;
pub mod intent;
pub mod schema;
