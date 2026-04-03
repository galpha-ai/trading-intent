use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use super::router::AppState;
use crate::dispatch;
use crate::intent::{self, ValidatedIntent};
use crate::schema::{self, template};

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IntentRequest {
    pub intent: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Serialize)]
pub struct DispatcherInfo {
    pub intent_type: Option<String>,
    pub chain_id: Option<String>,
    pub endpoint: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(code: &str, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorDetail {
                code: code.into(),
                message: msg.into(),
            },
        }),
    )
}

fn err_status(
    status: StatusCode,
    code: &str,
    msg: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorDetail {
                code: code.into(),
                message: msg.into(),
            },
        }),
    )
}

/// Collect all shorthands from schemas matching the intent type (or all if unknown).
fn collect_shorthands(
    state: &AppState,
    intent_type: Option<&str>,
) -> Vec<crate::schema::loader::XmlShorthand> {
    if let Some(t) = intent_type {
        state
            .registry
            .get(t)
            .map(|s| s.xml_shorthands.clone())
            .unwrap_or_default()
    } else {
        // If type unknown at parse time, collect all shorthands
        state
            .registry
            .list()
            .iter()
            .flat_map(|s| s.xml_shorthands.clone())
            .collect()
    }
}

/// Parse XML, validate against schema, return ValidatedIntent.
fn parse_and_validate(
    xml: &str,
    state: &AppState,
) -> Result<ValidatedIntent, (StatusCode, Json<ErrorResponse>)> {
    // Parse with all shorthands (we don't know the type yet)
    let shorthands = collect_shorthands(state, None);
    let parsed = intent::parse_xml(xml, &shorthands)
        .map_err(|e| err("PARSE_ERROR", format!("Invalid XML: {e}")))?;

    // Extract type and chain_id
    let intent_type = parsed
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| err("PARSE_ERROR", "Missing <type> element"))?
        .to_string();

    let chain_id = parsed
        .get("chain_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| err("PARSE_ERROR", "Missing <chain_id> element"))?
        .to_string();

    // Validate against schema
    schema::validate(&parsed, &state.registry)
        .map_err(|e| err("VALIDATION_ERROR", e.to_string()))?;

    Ok(ValidatedIntent {
        intent_id: Uuid::new_v4().to_string(),
        intent_type,
        chain_id,
        payload: parsed,
        raw_xml: xml.to_string(),
        received_at: Utc::now(),
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/dispatch — parse, validate, dispatch.
pub async fn dispatch_intent(
    State(state): State<AppState>,
    Json(req): Json<IntentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let intent = parse_and_validate(&req.intent, &state)?;

    let resp = dispatch::dispatch(intent, &state.config.dispatchers, &state.http_client)
        .await
        .map_err(|e| {
            let (status, code) = match &e {
                dispatch::DispatchError::NoMatch { .. } => (StatusCode::NOT_FOUND, "NO_DISPATCHER"),
                dispatch::DispatchError::ExecutorError { status, .. } => (
                    StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY),
                    "EXECUTOR_ERROR",
                ),
                dispatch::DispatchError::Timeout { .. } => (StatusCode::GATEWAY_TIMEOUT, "TIMEOUT"),
                dispatch::DispatchError::RequestFailed(_) => {
                    (StatusCode::BAD_GATEWAY, "DISPATCH_FAILED")
                }
            };
            error!(error = %e, "Dispatch failed");
            err_status(status, code, e.to_string())
        })?;

    Ok(Json(resp.executor_response))
}

/// POST /api/v1/validate — parse and validate, return structured JSON.
pub async fn validate_intent(
    State(state): State<AppState>,
    Json(req): Json<IntentRequest>,
) -> Result<Json<ValidatedIntent>, (StatusCode, Json<ErrorResponse>)> {
    let intent = parse_and_validate(&req.intent, &state)?;
    Ok(Json(intent))
}

/// POST /api/v1/parse — parse XML only, no schema validation.
pub async fn parse_intent(
    State(state): State<AppState>,
    Json(req): Json<IntentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let shorthands = collect_shorthands(&state, None);
    let parsed = intent::parse_xml(&req.intent, &shorthands)
        .map_err(|e| err("PARSE_ERROR", format!("Invalid XML: {e}")))?;
    Ok(Json(parsed))
}

/// GET /api/v1/templates — list available intent templates.
pub async fn list_templates(State(state): State<AppState>) -> Json<Vec<template::TemplateSummary>> {
    Json(template::list_templates(&state.registry))
}

/// GET /api/v1/templates/:intent_type — get template for a specific type.
pub async fn get_template(
    State(state): State<AppState>,
    Path(intent_type): Path<String>,
) -> Result<Json<template::TemplateInfo>, (StatusCode, Json<ErrorResponse>)> {
    let schema = state.registry.get(&intent_type).ok_or_else(|| {
        err_status(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            format!("No schema for intent type: {intent_type}"),
        )
    })?;
    Ok(Json(template::get_template(schema)))
}

/// GET /api/v1/dispatchers
pub async fn list_dispatchers(State(state): State<AppState>) -> Json<Vec<DispatcherInfo>> {
    Json(
        state
            .config
            .dispatchers
            .iter()
            .map(|d| DispatcherInfo {
                intent_type: d.match_rule.intent_type.clone(),
                chain_id: d.match_rule.chain_id.clone(),
                endpoint: d.endpoint.clone(),
            })
            .collect(),
    )
}

/// GET /health
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "tim",
    })
}
