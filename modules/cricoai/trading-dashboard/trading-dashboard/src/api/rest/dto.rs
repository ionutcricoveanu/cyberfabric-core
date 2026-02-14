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

/// Daily P&L aggregation.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DailyPnlDto {
    pub date: String,
    pub trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub daily_pnl: f64,
}

/// Wrapper for daily P&L list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DailyPnlListDto {
    pub daily_pnl: Vec<DailyPnlDto>,
}

/// A recent trade (completed sell order with P&L).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RecentTradeDto {
    pub timestamp: Option<String>,
    pub symbol: Option<String>,
    pub pnl: Option<f64>,
    pub client_order_id: Option<String>,
}

/// Wrapper for recent trades list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RecentTradesListDto {
    pub trades: Vec<RecentTradeDto>,
}

/// Aggregate trading statistics.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TradingStatisticsDto {
    pub total_pnl: f64,
    pub number_of_trades: i64,
    pub unique_days: i64,
    pub avg_pnl_per_trade: f64,
    pub avg_pnl_per_day: f64,
    pub trades_per_day: f64,
    pub available_usdc: f64,
    pub available_bnb: f64,
    pub pairs_evolution_day: PairsEvolutionDto,
    pub pairs_evolution_hour: PairsEvolutionDto,
}

/// Pair evolution counts (positive/negative).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PairsEvolutionDto {
    pub positive: i64,
    pub negative: i64,
}

/// Top trading pair by P&L.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TopPairDto {
    pub symbol: String,
    pub total_trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub total_pnl: f64,
    pub win_rate: f64,
}

/// Wrapper for top pairs list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TopPairsListDto {
    pub pairs: Vec<TopPairDto>,
}

/// Hourly asset value data point.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AssetValuePointDto {
    pub date: String,
    pub value: f64,
}

/// Total assets value response with 24h hourly data.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TotalAssetsValueDto {
    pub hourly_24h: Vec<AssetValuePointDto>,
}

/// Open buy order.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct OpenOrderDto {
    pub id: i32,
    pub symbol: Option<String>,
    pub client_order_id: Option<String>,
    pub transact_time: Option<String>,
    pub price: Option<f64>,
    pub qty: Option<f64>,
    pub status: Option<String>,
    pub est_profit_percent: Option<f64>,
    pub highest_percentage_since_buy: Option<f64>,
    pub lowest_percentage_since_buy: Option<f64>,
}

/// Wrapper for open orders list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct OpenOrdersListDto {
    pub orders: Vec<OpenOrderDto>,
}

/// Account balance overview.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AccountBalancesDto {
    pub available_usdc: f64,
    pub total_asset_val: f64,
    pub total_value: f64,
    pub invested: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
}
