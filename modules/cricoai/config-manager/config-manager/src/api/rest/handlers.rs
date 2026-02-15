use axum::Extension;
use axum::extract::{Path, Query};
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::ConfigManagerService;

#[derive(Debug, Deserialize)]
pub struct EnvQuery {
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_env() -> String {
    "production".to_string()
}

#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_percent")]
    pub percent: f64,
}

fn default_percent() -> f64 {
    1.0
}

// ================================================================
// Config endpoints
// ================================================================

pub(crate) async fn get_config(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Query(query): Query<EnvQuery>,
) -> ApiResult<JsonBody<ConfigResponse>> {
    let result = svc.get_config(&ctx, &query.env).await?;
    Ok(Json(result))
}

pub(crate) async fn update_config(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Query(query): Query<EnvQuery>,
    Json(body): Json<ConfigUpdateRequest>,
) -> ApiResult<JsonBody<ConfigUpdateResponse>> {
    let result = svc.update_config(&ctx, &query.env, &body.section, &body.config).await?;
    Ok(Json(result))
}

pub(crate) async fn get_config_raw(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Query(query): Query<EnvQuery>,
) -> ApiResult<JsonBody<RawConfigResponse>> {
    let result = svc.get_config_raw(&ctx, &query.env).await?;
    Ok(Json(result))
}

pub(crate) async fn update_config_raw(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Query(query): Query<EnvQuery>,
    Json(body): Json<RawConfigUpdateRequest>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let result = svc.update_config_raw(&ctx, &query.env, &body.yaml).await?;
    Ok(Json(result))
}

// ================================================================
// Trade pairs endpoints
// ================================================================

pub(crate) async fn get_trade_pairs(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
) -> ApiResult<JsonBody<TradePairsListDto>> {
    let result = svc.get_trade_pairs(&ctx).await?;
    Ok(Json(result))
}

pub(crate) async fn get_signals(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Path(symbol): Path<String>,
    Query(query): Query<SignalQuery>,
) -> ApiResult<JsonBody<SignalsResponse>> {
    let result = svc.get_signals(&ctx, &symbol, query.percent).await?;
    Ok(Json(result))
}

pub(crate) async fn update_profit_percent(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Path(symbol): Path<String>,
    Json(body): Json<ProfitPercentUpdateRequest>,
) -> ApiResult<JsonBody<ProfitPercentUpdateResponse>> {
    let result = svc.update_profit_percent(&ctx, &symbol, body.percent).await?;
    Ok(Json(result))
}

// ================================================================
// Restart endpoint
// ================================================================

pub(crate) async fn restart_container(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ConfigManagerService>>,
    Query(query): Query<EnvQuery>,
) -> ApiResult<JsonBody<RestartResponse>> {
    let result = svc.restart_container(&ctx, &query.env).await?;
    Ok(Json(result))
}
