use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};
use tracing::info;

use super::dto;
use super::handlers;
use crate::domain::service::ModelDashboardService;

// ── Local auth enums ─────────

pub(crate) enum Resource { Models }

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self { Self::Models => "models" }
    }
}

impl AuthReqResource for Resource {}

pub(crate) enum Action { Read }

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self { Self::Read => "read" }
    }
}

impl AuthReqAction for Action {}

pub(crate) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &str { "" }
}

impl LicenseFeature for License {}

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<ModelDashboardService>,
) -> Router {
    info!("Registering model-dashboard REST routes");

    // GET /model-dashboard/v1/summary
    router = OperationBuilder::get("/model-dashboard/v1/summary")
        .operation_id("model_dashboard.get_summary")
        .summary("Get model training summary")
        .description("Returns aggregated training metrics by interval with promotion rates")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_summary)
        .json_response_with_schema::<dto::ModelSummaryResponse>(openapi, http::StatusCode::OK, "Summary data")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/training-history
    router = OperationBuilder::get("/model-dashboard/v1/training-history")
        .operation_id("model_dashboard.get_training_history")
        .summary("Get model training history")
        .description("Returns paginated training history with filtering")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .query_param("symbol", false, "Filter by symbol")
        .query_param("promoted_only", false, "Show only promoted models")
        .query_param("rejected_only", false, "Show only rejected models")
        .query_param("limit", false, "Page size (max 500)")
        .query_param("offset", false, "Page offset")
        .handler(handlers::get_training_history)
        .json_response_with_schema::<dto::TrainingHistoryResponse>(openapi, http::StatusCode::OK, "Training history")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/rejection-reasons
    router = OperationBuilder::get("/model-dashboard/v1/rejection-reasons")
        .operation_id("model_dashboard.get_rejection_reasons")
        .summary("Get rejection reason breakdown")
        .description("Returns rejection reason counts grouped by interval")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_rejection_reasons)
        .json_response_with_schema::<dto::RejectionReasonsResponse>(openapi, http::StatusCode::OK, "Rejection reasons")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/symbol-status
    router = OperationBuilder::get("/model-dashboard/v1/symbol-status")
        .operation_id("model_dashboard.get_symbol_status")
        .summary("Get current model status per symbol")
        .description("Shows latest training result and model age for each symbol")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .handler(handlers::get_symbol_status)
        .json_response_with_schema::<dto::SymbolStatusResponse>(openapi, http::StatusCode::OK, "Symbol statuses")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/training-timeline
    router = OperationBuilder::get("/model-dashboard/v1/training-timeline")
        .operation_id("model_dashboard.get_training_timeline")
        .summary("Get training activity timeline")
        .description("Aggregates training counts and promotion rates by day for charts")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_training_timeline)
        .json_response_with_schema::<dto::TrainingTimelineResponse>(openapi, http::StatusCode::OK, "Timeline data")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/performance-distribution
    router = OperationBuilder::get("/model-dashboard/v1/performance-distribution")
        .operation_id("model_dashboard.get_performance_distribution")
        .summary("Get performance metric distributions")
        .description("Returns win rate, return, sharpe, etc. distributions for chart rendering")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", false, "Interval filter: all, 1m, 5m")
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .query_param("promoted_only", false, "Show only promoted models (default: true)")
        .handler(handlers::get_performance_distribution)
        .json_response_with_schema::<dto::PerformanceDistributionResponse>(openapi, http::StatusCode::OK, "Distribution data")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/calibration-metrics
    router = OperationBuilder::get("/model-dashboard/v1/calibration-metrics")
        .operation_id("model_dashboard.get_calibration_metrics")
        .summary("Get model calibration metrics")
        .description("Prediction confidence vs actual accuracy by symbol and model version")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_calibration_metrics)
        .json_response_with_schema::<dto::CalibrationMetricsResponse>(openapi, http::StatusCode::OK, "Calibration metrics")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/regime-analysis
    router = OperationBuilder::get("/model-dashboard/v1/regime-analysis")
        .operation_id("model_dashboard.get_regime_analysis")
        .summary("Get regime-based performance analysis")
        .description("Analyze prediction performance by market regime and conflict status")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_regime_analysis)
        .json_response_with_schema::<dto::RegimeAnalysisResponse>(openapi, http::StatusCode::OK, "Regime analysis")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /model-dashboard/v1/model-age-impact
    router = OperationBuilder::get("/model-dashboard/v1/model-age-impact")
        .operation_id("model_dashboard.get_model_age_impact")
        .summary("Get model age impact analysis")
        .description("Analyze how model age affects prediction accuracy and calibration")
        .tag("model-dashboard")
        .require_auth(&Resource::Models, &Action::Read)
        .require_license_features::<License>([])
        .query_param("time_range", false, "Time range: 24h, 7d, 30d, all")
        .handler(handlers::get_model_age_impact)
        .json_response_with_schema::<dto::AgeImpactResponse>(openapi, http::StatusCode::OK, "Age impact data")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    info!("Model-dashboard REST routes registered successfully");

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
