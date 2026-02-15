use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::AgentTradeImpactService;

#[derive(Debug, Deserialize)]
pub struct ImpactQuery {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_days")]
    pub days: i64,
    pub pair: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EffectivenessQuery {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_effectiveness_days")]
    pub days: i64,
}

fn default_env() -> String { "production".to_string() }
fn default_days() -> i64 { 7 }
fn default_effectiveness_days() -> i64 { 30 }

pub(crate) async fn get_buy_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentTradeImpactService>>,
    Query(q): Query<ImpactQuery>,
) -> ApiResult<JsonBody<BuyImpactResponse>> {
    let days = q.days.min(90).max(1);
    let resp = svc.get_buy_impact(&ctx, &q.env, days, q.pair.as_deref()).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_sell_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentTradeImpactService>>,
    Query(q): Query<ImpactQuery>,
) -> ApiResult<JsonBody<SellImpactResponse>> {
    let days = q.days.min(90).max(1);
    let resp = svc.get_sell_impact(&ctx, &q.env, days, q.pair.as_deref()).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_effectiveness(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentTradeImpactService>>,
    Query(q): Query<EffectivenessQuery>,
) -> ApiResult<JsonBody<EffectivenessResponse>> {
    let days = q.days.min(365).max(1);
    let resp = svc.get_effectiveness(&ctx, &q.env, days).await?;
    Ok(Json(resp))
}
