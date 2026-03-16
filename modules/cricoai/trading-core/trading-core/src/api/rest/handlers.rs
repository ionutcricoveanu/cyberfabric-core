//! REST API handlers for trading-core module

use axum::extract::Path;
use axum::Extension;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use std::sync::Arc;

use crate::domain::service::TradingCoreService;
use super::dto::*;
use super::error;

pub type BotStatusResponse = super::dto::BotStatusResponse;
pub type ActionResponse = super::dto::ActionResponse;
pub type PositionResponse = Option<super::dto::PositionInfo>;
pub type PositionsResponse = Vec<super::dto::PositionInfo>;
pub type OrderHistoryResponse = Vec<super::dto::OrderHistoryEntry>;
pub type TradingSummaryResponse = super::dto::TradingSummary;
pub type HealthResponse = super::dto::HealthResponse;

/// Get bot status
pub async fn get_status(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
) -> ApiResult<JsonBody<BotStatusResponse>> {
    let status = service.get_status(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(BotStatusResponse {
        running: status.running,
        paused: status.paused,
        open_positions: status.open_positions,
        last_trade_time: status.last_trade_time,
    }))
}

/// Pause the trading bot
pub async fn pause_trading(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
    Json(payload): Json<PauseRequest>,
) -> ApiResult<JsonBody<ActionResponse>> {
    service.pause(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Trading paused{}",
            payload.reason.as_ref().map(|r| format!(" ({})", r)).unwrap_or_default()
        ),
    }))
}

/// Resume the trading bot
pub async fn resume_trading(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
    Json(payload): Json<ResumeRequest>,
) -> ApiResult<JsonBody<ActionResponse>> {
    service.resume(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Trading resumed{}",
            payload.reason.as_ref().map(|r| format!(" ({})", r)).unwrap_or_default()
        ),
    }))
}

/// Get open positions
pub async fn get_positions(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
) -> ApiResult<JsonBody<Vec<PositionInfo>>> {
    let positions = service.get_positions(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(positions))
}

/// Get position details by symbol
pub async fn get_position(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
    Path(symbol): Path<String>,
) -> ApiResult<JsonBody<Option<PositionInfo>>> {
    let position = service.get_position(&ctx, &symbol).await.map_err(error::to_problem)?;
    Ok(Json(position))
}

/// Get order history
pub async fn get_order_history(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
) -> ApiResult<JsonBody<Vec<OrderHistoryEntry>>> {
    let orders = service.get_order_history(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(orders))
}

/// Get trading summary
pub async fn get_trading_summary(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<TradingCoreService>>,
) -> ApiResult<JsonBody<TradingSummary>> {
    let summary = service.get_trading_summary(&ctx).await.map_err(error::to_problem)?;
    Ok(Json(summary))
}

/// Health check endpoint
pub async fn health(
    Extension(service): Extension<Arc<TradingCoreService>>,
) -> ApiResult<JsonBody<HealthResponse>> {
    let status = if service.is_healthy().await {
        "healthy"
    } else {
        "unhealthy"
    };

    Ok(Json(HealthResponse {
        status: status.to_string(),
        service: "trading-core".to_string(),
    }))
}
