use trading_dashboard_sdk::models::StatsSummary;

/// REST DTO for trading statistics summary.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StatsSummaryDto {
    pub total_trades: i64,
    pub open_orders: i64,
    pub total_pnl: f64,
    pub total_asset_value: f64,
    pub available_usdc: f64,
    pub trading_enabled: bool,
}

impl From<StatsSummary> for StatsSummaryDto {
    fn from(s: StatsSummary) -> Self {
        Self {
            total_trades: s.total_trades,
            open_orders: s.open_orders,
            total_pnl: s.total_pnl,
            total_asset_value: s.total_asset_value,
            available_usdc: s.available_usdc,
            trading_enabled: s.trading_enabled,
        }
    }
}
