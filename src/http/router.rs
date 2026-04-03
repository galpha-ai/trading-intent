use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use super::handlers;
use crate::config::Config;
use crate::schema::SchemaRegistry;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub registry: Arc<SchemaRegistry>,
    pub http_client: reqwest::Client,
}

pub fn build(config: Config, registry: SchemaRegistry) -> Router {
    let state = AppState {
        config,
        registry: Arc::new(registry),
        http_client: reqwest::Client::new(),
    };

    Router::new()
        .route("/api/v1/dispatch", post(handlers::dispatch_intent))
        .route("/api/v1/validate", post(handlers::validate_intent))
        .route("/api/v1/parse", post(handlers::parse_intent))
        .route("/api/v1/templates", get(handlers::list_templates))
        .route(
            "/api/v1/templates/{intent_type}",
            get(handlers::get_template),
        )
        .route("/api/v1/dispatchers", get(handlers::list_dispatchers))
        .route("/health", get(handlers::health))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state)
}
