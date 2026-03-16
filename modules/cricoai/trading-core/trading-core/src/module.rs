//! ModKit module definition and lifecycle management for trading-core

use std::sync::Arc;
use tracing::{info, warn, debug};
use modkit::{Module, ModuleCtx, RestApiCapability};
use modkit::api::OpenApiRegistry;
use axum::Router;
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;
use modkit_security::SecurityContext;
use std::time::Instant;
use async_trait::async_trait;

use crate::config::TradingCoreConfig;
use crate::domain::service::TradingCoreService;
use crate::domain::local_client::LocalClient;
use crate::domain::{BuyManager, SellManager, BinanceApiClient, ModuleClients};
use crate::domain::models::{BuySignal, Quote, AccountBalance};
use rust_decimal::prelude::ToPrimitive;
use crate::domain::persistence;
use crate::api::rest::routes::register_routes;
use modkit_db::Db;
use trading_core_sdk::TradingCoreApi;

pub type TradingCoreModule = TradingCore;

/// Main trading-core module implementation
#[modkit::module(
    name = "trading-core",
    capabilities = [stateful, rest],
    lifecycle(entry = "run_trading_loop", stop_timeout = "30s")
)]
pub struct TradingCore {
    config: arc_swap::ArcSwapOption<TradingCoreConfig>,

    /// Service instance
    service: arc_swap::ArcSwapOption<TradingCoreService>,

    /// Buy strategy manager
    buy_manager: arc_swap::ArcSwapOption<BuyManager>,

    /// Sell strategy manager
    sell_manager: arc_swap::ArcSwapOption<SellManager>,

    /// Binance API client
    binance_client: arc_swap::ArcSwapOption<BinanceApiClient>,

    /// Other module clients
    module_clients: arc_swap::ArcSwapOption<ModuleClients>,

    /// Database connection for persistence
    db_binance: arc_swap::ArcSwapOption<Db>,

    /// Track last execution times for staggered strategies
    last_buy_execution: Arc<TokioMutex<Instant>>,
    last_sell_execution: Arc<TokioMutex<Instant>>,
}

impl Default for TradingCore {
    fn default() -> Self {
        Self {
            config: arc_swap::ArcSwapOption::from(None),
            service: arc_swap::ArcSwapOption::from(None),
            buy_manager: arc_swap::ArcSwapOption::from(None),
            sell_manager: arc_swap::ArcSwapOption::from(None),
            binance_client: arc_swap::ArcSwapOption::from(None),
            module_clients: arc_swap::ArcSwapOption::from(None),
            db_binance: arc_swap::ArcSwapOption::from(None),
            last_buy_execution: Arc::new(TokioMutex::new(Instant::now())),
            last_sell_execution: Arc::new(TokioMutex::new(Instant::now())),
        }
    }
}

impl Clone for TradingCore {
    fn clone(&self) -> Self {
        Self {
            config: arc_swap::ArcSwapOption::new(self.config.load().as_ref().map(Clone::clone)),
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
            buy_manager: arc_swap::ArcSwapOption::new(self.buy_manager.load().as_ref().map(Clone::clone)),
            sell_manager: arc_swap::ArcSwapOption::new(self.sell_manager.load().as_ref().map(Clone::clone)),
            binance_client: arc_swap::ArcSwapOption::new(self.binance_client.load().as_ref().map(Clone::clone)),
            module_clients: arc_swap::ArcSwapOption::new(self.module_clients.load().as_ref().map(Clone::clone)),
            db_binance: arc_swap::ArcSwapOption::new(self.db_binance.load().as_ref().map(Clone::clone)),
            last_buy_execution: Arc::clone(&self.last_buy_execution),
            last_sell_execution: Arc::clone(&self.last_sell_execution),
        }
    }
}

#[async_trait]
impl Module for TradingCore {
    /// Initialize the trading-core module
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing trading-core module");

        let cfg: Arc<TradingCoreConfig> = Arc::new(ctx.config()?);
        self.config.store(Some(cfg.clone()));

        // Connect to Binance database
        info!("Connecting to Binance database");
        let binance_db = Arc::new(
            modkit_db::connect_db(&cfg.binance_dsn, modkit_db::ConnectOpts::default())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Binance database: {e}"))?,
        );
        self.db_binance.store(Some(binance_db.clone()));

        // Create service instance
        let service = Arc::new(TradingCoreService::new((*cfg).clone()).await?);
        self.service.store(Some(service.clone()));

        // Create strategy managers
        let buy_manager = BuyManager::new(cfg.clone(), binance_db.clone());
        self.buy_manager.store(Some(Arc::new(buy_manager)));

        let sell_manager = SellManager::new(cfg.clone(), binance_db.clone());
        self.sell_manager.store(Some(Arc::new(sell_manager)));

        // Create Binance API client
        let binance_client = Arc::new(BinanceApiClient::new(cfg.clone()));
        self.binance_client.store(Some(binance_client.clone()));

        // Test Binance connectivity
        info!("Testing Binance connectivity");
        match binance_client.ping().await {
            Ok(_) => info!("✓ Binance connectivity OK"),
            Err(e) => warn!("⚠ Binance connectivity check failed: {}", e),
        }

        // Create module clients
        let module_clients = ModuleClients::new(ctx.client_hub(), cfg.clone());
        self.module_clients.store(Some(Arc::new(module_clients)));

        // Register local client with ClientHub
        let local_client = LocalClient::new(service.clone());
        ctx.client_hub()
            .register::<dyn TradingCoreApi>(Arc::new(local_client));

        info!("✓ Trading-core module initialized successfully");
        Ok(())
    }

}

impl RestApiCapability for TradingCore {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<Router> {
        info!("Registering trading-core REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = register_routes(router, openapi, service);

        info!("Trading-core REST routes registered successfully");
        Ok(router)
    }
}

impl TradingCore {
    /// Entry point for the background trading loop
    pub(crate) async fn run_trading_loop(
        self: Arc<Self>,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        info!("Starting trading loop");

        let config = self
            .config
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Config not initialized"))?
            .clone();

        let buy_manager = self
            .buy_manager
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BuyManager not initialized"))?
            .clone();

        let sell_manager = self
            .sell_manager
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SellManager not initialized"))?
            .clone();

        let binance_client = self
            .binance_client
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Binance client not initialized"))?
            .clone();

        let module_clients = self
            .module_clients
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Module clients not initialized"))?
            .clone();

        let db_binance = self
            .db_binance
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?
            .clone();

        let buy_interval = std::time::Duration::from_secs(config.buy_strategy_interval_secs);
        let sell_interval = std::time::Duration::from_secs(config.sell_strategy_interval_secs);
        let main_interval = std::time::Duration::from_secs(config.main_loop_interval_secs);

        let ctx = SecurityContext::anonymous();
        let mut cycle_count = 0u64;
        
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("Trading loop cancellation requested");
                    break;
                }
                
                // Main loop runs every configured interval
                _ = tokio::time::sleep(main_interval) => {
                    cycle_count += 1;
                    let cycle_start = Instant::now();
                    
                    info!("[Cycle {}] Starting trading cycle", cycle_count);

                    let now = Instant::now();
                    let should_buy = {
                        let mut last = self.last_buy_execution.lock().await;
                        if now.duration_since(*last) >= buy_interval {
                            *last = now;
                            true
                        } else {
                            false
                        }
                    };

                    let should_sell = {
                        let mut last = self.last_sell_execution.lock().await;
                        if now.duration_since(*last) >= sell_interval {
                            *last = now;
                            true
                        } else {
                            false
                        }
                    };

                    if !should_buy && !should_sell {
                        debug!("[Cycle {}] No strategy due this cycle", cycle_count);
                    } else {
                        let trading_config = match module_clients.get_trading_config(&ctx).await {
                            Ok(cfg) => cfg,
                            Err(err) => {
                                warn!("[Cycle {}] Failed to fetch trading config: {}", cycle_count, err);
                                continue;
                            }
                        };

                        if should_sell {
                            match sell_manager.execute_sell_strategy(&ctx, &binance_client).await {
                                Ok(orders) => {
                                    for order in orders {
                                        let entry_price = order.price;
                                        match binance_client
                                            .place_limit_order(&order.symbol, &order.side, order.quantity, order.price)
                                            .await
                                        {
                                            Ok(placed) => {
                                                info!("Sell order placed: id={} symbol={} status={}",
                                                      placed.order_id, placed.symbol, placed.status);
                                                let pnl_val = (placed.price - entry_price)
                                                    .to_f64()
                                                    .unwrap_or(0.0)
                                                    * placed.orig_qty;
                                                if let Err(e) = persistence::record_sell_order(
                                                    &db_binance,
                                                    &placed,
                                                    None,
                                                    entry_price,
                                                    pnl_val,
                                                ).await {
                                                    warn!("Failed to persist sell order for {}: {}", order.symbol, e);
                                                }
                                            }
                                            Err(err) => warn!("Sell order failed for {}: {}", order.symbol, err),
                                        }
                                    }
                                }
                                Err(err) => warn!("Sell strategy failed: {}", err),
                            }
                        }

                        if should_buy {
                            let balances = match binance_client.get_account_balance().await {
                                Ok(balances) => balances,
                                Err(err) => {
                                    warn!("Failed to fetch account balances: {}", err);
                                    Vec::new()
                                }
                            };

                            let account_balance = balances
                                .iter()
                                .find(|b| b.asset == "USDT")
                                .cloned()
                                .unwrap_or(AccountBalance {
                                    asset: "USDT".to_string(),
                                    free: rust_decimal::Decimal::ZERO,
                                    locked: rust_decimal::Decimal::ZERO,
                                    total: rust_decimal::Decimal::ZERO,
                                });

                            for symbol in trading_config.symbols {
                                let market_data = match module_clients.get_market_data(&ctx, &symbol).await {
                                    Ok(data) => data,
                                    Err(err) => {
                                        warn!("Failed to fetch market data for {}: {}", symbol, err);
                                        continue;
                                    }
                                };

                                let ml_prediction = match module_clients
                                    .get_ml_prediction(&ctx, &symbol, &config.kline_interval)
                                    .await
                                {
                                    Ok(prediction) => prediction,
                                    Err(err) => {
                                        warn!("Failed to fetch ML prediction for {}: {}", symbol, err);
                                        continue;
                                    }
                                };

                                let agent_decision = match module_clients.get_agent_decision(&ctx, &symbol).await {
                                    Ok(decision) => decision,
                                    Err(err) => {
                                        warn!("Failed to fetch agent decision for {}: {}", symbol, err);
                                        continue;
                                    }
                                };

                                let combined_confidence = (ml_prediction.confidence + agent_decision.confidence) / 2.0;
                                let action = if (ml_prediction.direction == "BUY" || agent_decision.action == "BUY")
                                    && combined_confidence >= trading_config.buy_signal_threshold
                                {
                                    "BUY"
                                } else {
                                    "HOLD"
                                };

                                let buy_signal = BuySignal {
                                    symbol: symbol.clone(),
                                    action: action.to_string(),
                                    confidence: combined_confidence,
                                    predicted_change_pct: ml_prediction.predicted_change_pct,
                                    timestamp: chrono::Utc::now().naive_utc(),
                                };

                                let quote = Quote {
                                    symbol: symbol.clone(),
                                    bid: market_data.bid_price,
                                    ask: market_data.ask_price,
                                    last_trade_price: market_data.current_price,
                                    timestamp: chrono::Utc::now().naive_utc(),
                                };

                                match buy_manager
                                    .execute_buy_strategy(&ctx, &symbol, buy_signal, &account_balance, &quote)
                                    .await
                                {
                                    Ok(Some(order)) => {
                                        match binance_client
                                            .place_limit_order(&order.symbol, &order.side, order.quantity, order.price)
                                            .await
                                        {
                                            Ok(placed) => {
                                                info!("Buy order placed: id={} symbol={} status={}",
                                                      placed.order_id, placed.symbol, placed.status);
                                                if let Err(e) = persistence::record_buy_order(
                                                    &db_binance,
                                                    &placed,
                                                ).await {
                                                    warn!("Failed to persist buy order for {}: {}", order.symbol, e);
                                                }
                                            }
                                            Err(err) => warn!("Buy order failed for {}: {}", order.symbol, err),
                                        }
                                    }
                                    Ok(None) => {}
                                    Err(err) => warn!("Buy strategy failed for {}: {}", symbol, err),
                                }
                            }
                        }
                    }

                    // TODO: Implement full trading cycle:
                    // 1. Check if trading is enabled
                    // 2. Get all trading pairs from config
                    // For each pair:
                    //   a. Get current market data (via market-data module)
                    //   b. Get ML prediction (via ml-service gRPC)
                    //   c. Get agent decision (via market-intelligence module)
                    //   d. Execute buy strategy (if buy interval elapsed)
                    //   e. Execute sell strategy (if sell interval elapsed)
                    // 3. Record cycle metrics and P&L
                    // 4. Check risk limits and daily loss limit

                    let elapsed = cycle_start.elapsed();
                    let sleep_time = if elapsed < main_interval {
                        (main_interval - elapsed).as_secs()
                    } else {
                        1
                    };
                    
                    info!("[Cycle {}] Cycle complete (elapsed: {}s, sleeping {}s)", 
                          cycle_count, elapsed.as_secs(), sleep_time);
                }
            }
        }

        info!("Trading loop stopped after {} cycles", cycle_count);
        Ok(())
    }
}
