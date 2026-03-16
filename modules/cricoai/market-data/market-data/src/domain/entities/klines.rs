use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use modkit_db_macros::Scopable;

/// Base entity for klines tables (klines_1m, klines_5m, klines_15m, klines_1h, klines_4h).
/// This is an unrestricted entity (no tenant scoping).
///
/// All klines tables share the same schema, only differing by table name.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "klines_1m")]
#[secure(unrestricted)]
pub struct Model {
    /// Kline open time (UTC) - primary key part 1
    #[sea_orm(primary_key, auto_increment = false)]
    pub open_time: DateTime<Utc>,

    /// Trading pair symbol (e.g., "BTCUSDT") - primary key part 2
    #[sea_orm(primary_key, auto_increment = false)]
    pub symbol: String,

    /// Kline close time (UTC)
    pub close_time: Option<DateTime<Utc>>,

    /// Open price
    pub open: Option<f64>,

    /// Close price
    pub close: Option<f64>,

    /// High price
    pub high: Option<f64>,

    /// Low price
    pub low: Option<f64>,

    /// Base asset volume
    pub volume: Option<f64>,

    /// Quote asset volume
    pub quote_asset_volume: Option<f64>,

    /// Number of trades
    pub number_of_trades: Option<i32>,

    /// Taker buy base asset volume
    pub taker_buy_base_asset_volume: Option<f64>,

    /// Taker buy quote asset volume
    pub taker_buy_quote_asset_volume: Option<f64>,

    /// Whether this kline is closed (complete)
    pub kline_closed: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Helper macro to create interval-specific entities
macro_rules! define_kline_entity {
    ($module_name:ident, $table_name:literal) => {
        pub mod $module_name {
            use super::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
            #[sea_orm(table_name = $table_name)]
            #[secure(unrestricted)]
            pub struct Model {
                #[sea_orm(primary_key, auto_increment = false)]
                pub open_time: DateTime<Utc>,

                #[sea_orm(primary_key, auto_increment = false)]
                pub symbol: String,

                pub close_time: Option<DateTime<Utc>>,
                pub open: Option<f64>,
                pub close: Option<f64>,
                pub high: Option<f64>,
                pub low: Option<f64>,
                pub volume: Option<f64>,
                pub quote_asset_volume: Option<f64>,
                pub number_of_trades: Option<i32>,
                pub taker_buy_base_asset_volume: Option<f64>,
                pub taker_buy_quote_asset_volume: Option<f64>,
                pub kline_closed: Option<bool>,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}

            /// Convert from SDK Kline to database ActiveModel
            impl From<&market_data_sdk::models::Kline> for ActiveModel {
                fn from(kline: &market_data_sdk::models::Kline) -> Self {
                    Self {
                        open_time: Set(kline.start_time),
                        symbol: Set(kline.symbol.clone()),
                        close_time: Set(Some(kline.close_time)),
                        open: Set(Some(kline.open_price)),
                        close: Set(Some(kline.close_price)),
                        high: Set(Some(kline.high_price)),
                        low: Set(Some(kline.low_price)),
                        volume: Set(Some(kline.base_asset_volume)),
                        quote_asset_volume: Set(Some(kline.quote_asset_volume)),
                        number_of_trades: Set(Some(kline.number_of_trades)),
                        taker_buy_base_asset_volume: Set(Some(kline.taker_buy_base_asset_volume)),
                        taker_buy_quote_asset_volume: Set(Some(kline.taker_buy_quote_asset_volume)),
                        kline_closed: Set(Some(kline.kline_closed)),
                    }
                }
            }

            /// Convert from database Model to SDK Kline
            impl From<Model> for market_data_sdk::models::Kline {
                fn from(model: Model) -> Self {
                    Self {
                        start_time: model.open_time,
                        close_time: model.close_time.unwrap_or(model.open_time),
                        symbol: model.symbol.clone(),
                        interval: $table_name.trim_start_matches("klines_").to_string(),
                        open_price: model.open.unwrap_or(0.0),
                        close_price: model.close.unwrap_or(0.0),
                        high_price: model.high.unwrap_or(0.0),
                        low_price: model.low.unwrap_or(0.0),
                        base_asset_volume: model.volume.unwrap_or(0.0),
                        quote_asset_volume: model.quote_asset_volume.unwrap_or(0.0),
                        number_of_trades: model.number_of_trades.unwrap_or(0),
                        taker_buy_base_asset_volume: model.taker_buy_base_asset_volume.unwrap_or(0.0),
                        taker_buy_quote_asset_volume: model.taker_buy_quote_asset_volume.unwrap_or(0.0),
                        kline_closed: model.kline_closed.unwrap_or(false),
                    }
                }
            }
        }
    };
}

// Define entities for each interval
define_kline_entity!(klines_1m, "klines_1m");
define_kline_entity!(klines_5m, "klines_5m");
define_kline_entity!(klines_15m, "klines_15m");
define_kline_entity!(klines_1h, "klines_1h");
define_kline_entity!(klines_4h, "klines_4h");

/// Convert from SDK Kline to database Model
impl From<&market_data_sdk::models::Kline> for ActiveModel {
    fn from(kline: &market_data_sdk::models::Kline) -> Self {
        Self {
            open_time: Set(kline.start_time),
            symbol: Set(kline.symbol.clone()),
            close_time: Set(Some(kline.close_time)),
            open: Set(Some(kline.open_price)),
            close: Set(Some(kline.close_price)),
            high: Set(Some(kline.high_price)),
            low: Set(Some(kline.low_price)),
            volume: Set(Some(kline.base_asset_volume)),
            quote_asset_volume: Set(Some(kline.quote_asset_volume)),
            number_of_trades: Set(Some(kline.number_of_trades)),
            taker_buy_base_asset_volume: Set(Some(kline.taker_buy_base_asset_volume)),
            taker_buy_quote_asset_volume: Set(Some(kline.taker_buy_quote_asset_volume)),
            kline_closed: Set(Some(kline.kline_closed)),
        }
    }
}

/// Convert from database Model to SDK Kline
impl From<Model> for market_data_sdk::models::Kline {
    fn from(model: Model) -> Self {
        Self {
            start_time: model.open_time,
            close_time: model.close_time.unwrap_or(model.open_time),
            symbol: model.symbol.clone(),
            interval: "".to_string(), // Filled in by caller based on table
            open_price: model.open.unwrap_or(0.0),
            close_price: model.close.unwrap_or(0.0),
            high_price: model.high.unwrap_or(0.0),
            low_price: model.low.unwrap_or(0.0),
            base_asset_volume: model.volume.unwrap_or(0.0),
            quote_asset_volume: model.quote_asset_volume.unwrap_or(0.0),
            number_of_trades: model.number_of_trades.unwrap_or(0),
            taker_buy_base_asset_volume: model.taker_buy_base_asset_volume.unwrap_or(0.0),
            taker_buy_quote_asset_volume: model.taker_buy_quote_asset_volume.unwrap_or(0.0),
            kline_closed: model.kline_closed.unwrap_or(false),
        }
    }
}
