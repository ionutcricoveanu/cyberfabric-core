use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use modkit_db::DBProvider;
use modkit_db::DbError;
use modkit_db::secure::{AccessScope, SecureEntityExt};
use modkit_security::SecurityContext;
use tenant_resolver_sdk::TenantResolverGatewayClient;
use tracing::info;

use crate::config::TradingDashboardConfig;
use crate::infra::storage::entity::{buy_orders, pnl, sell_orders, temps, total_asset_val};
use trading_dashboard_sdk::errors::TradingDashboardError;
use trading_dashboard_sdk::models::StatsSummary;

fn db_err(e: impl std::fmt::Display) -> TradingDashboardError {
    TradingDashboardError::Database(e.to_string())
}

/// Core service containing all trading dashboard business logic.
#[derive(Clone)]
pub struct TradingDashboardService {
    db: Arc<DBProvider<DbError>>,
    #[allow(dead_code)]
    resolver: Arc<dyn TenantResolverGatewayClient>,
    #[allow(dead_code)]
    config: TradingDashboardConfig,
}

impl TradingDashboardService {
    #[must_use]
    pub fn new(
        db: Arc<DBProvider<DbError>>,
        resolver: Arc<dyn TenantResolverGatewayClient>,
        config: TradingDashboardConfig,
    ) -> Self {
        Self {
            db,
            resolver,
            config,
        }
    }

    /// Get aggregated trading statistics summary.
    ///
    /// Queries across pnl, buy_orders, sell_orders, temps, and total_asset_val tables.
    /// All entities are `unrestricted` (no tenant scoping yet).
    pub async fn get_stats_summary(
        &self,
        ctx: &SecurityContext,
        _time_range: &str,
    ) -> Result<StatsSummary, TradingDashboardError> {
        info!(
            subject_id = %ctx.subject_id(),
            "Fetching trading stats summary"
        );

        let conn = self.db.conn().map_err(db_err)?;
        // Unrestricted entities use root scope — no tenant filtering applied
        // For unrestricted entities, scope value is ignored — use default
        let scope = AccessScope::default();

        // Count open buy orders
        let open_orders = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(db_err)?;

        // Count all completed trades (sell orders)
        let total_trades = sell_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(db_err)?;

        // Get latest temps row for available_usdc and trading_enabled
        let temps_row = temps::Entity::find()
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        // Get latest total asset value
        let asset_val = total_asset_val::Entity::find()
            .order_by_desc(total_asset_val::Column::RecordDate)
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        // Sum PnL — use secure .all() and sum in Rust (simpler than custom column projection through secure layer)
        let all_pnl = pnl::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_pnl: f64 = all_pnl.iter().filter_map(|p| p.pnl).sum();

        let (available_usdc, trading_enabled) = match temps_row {
            Some(t) => (
                t.available_usdc.unwrap_or(0.0),
                t.trading_enabled.unwrap_or(0) == 1,
            ),
            None => (0.0, false),
        };

        #[allow(clippy::cast_possible_wrap)]
        Ok(StatsSummary {
            total_trades: total_trades as i64,
            open_orders: open_orders as i64,
            total_pnl,
            total_asset_value: asset_val.map_or(0.0, |v| v.total_val.unwrap_or(0.0)),
            available_usdc,
            trading_enabled,
        })
    }
}
