//! Market Intelligence Service - orchestrates agent scheduling and decisions

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use market_intelligence_sdk::{AgentDecision, MarketIntelligenceError};
use modkit_db::Db;
use modkit_security::SecurityContext;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::config::MarketIntelligenceConfig;
use crate::infra::llm::OllamaClient;

/// Symbol tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolTier {
    Hot,
    Warm,
    Stable,
    Cold,
}

impl SymbolTier {
    fn name(&self) -> &str {
        match self {
            SymbolTier::Hot => "HOT",
            SymbolTier::Warm => "WARM",
            SymbolTier::Stable => "STABLE",
            SymbolTier::Cold => "COLD",
        }
    }
}

/// Last scheduled run for a tier
#[derive(Debug, Clone)]
struct TierSchedule {
    pub tier: SymbolTier,
    pub last_run: Option<chrono::DateTime<Utc>>,
    pub interval_secs: u64,
}

/// Market Intelligence Service
pub struct MarketIntelligenceService {
    db: Option<Arc<Db>>,
    config: Option<MarketIntelligenceConfig>,
    llm: Option<OllamaClient>,
    tier_schedules: Mutex<HashMap<SymbolTier, TierSchedule>>,
}

impl Default for MarketIntelligenceService {
    fn default() -> Self {
        Self {
            db: None,
            config: None,
            llm: None,
            tier_schedules: Mutex::new(HashMap::new()),
        }
    }
}

impl MarketIntelligenceService {
    /// Create a new market intelligence service
    pub fn new(db: Arc<Db>, config: MarketIntelligenceConfig) -> Self {
        let llm = OllamaClient::new(
            config.llm.ollama_url.clone(),
            config.llm.model_name.clone(),
        );

        let mut tier_schedules = HashMap::new();
        tier_schedules.insert(
            SymbolTier::Hot,
            TierSchedule {
                tier: SymbolTier::Hot,
                last_run: None,
                interval_secs: config.tier_schedule.hot_interval_minutes * 60,
            },
        );
        tier_schedules.insert(
            SymbolTier::Warm,
            TierSchedule {
                tier: SymbolTier::Warm,
                last_run: None,
                interval_secs: config.tier_schedule.warm_interval_minutes * 60,
            },
        );
        tier_schedules.insert(
            SymbolTier::Stable,
            TierSchedule {
                tier: SymbolTier::Stable,
                last_run: None,
                interval_secs: config.tier_schedule.stable_interval_minutes * 60,
            },
        );
        tier_schedules.insert(
            SymbolTier::Cold,
            TierSchedule {
                tier: SymbolTier::Cold,
                last_run: None,
                interval_secs: config.tier_schedule.cold_interval_minutes * 60,
            },
        );

        Self {
            db: Some(db),
            config: Some(config),
            llm: Some(llm),
            tier_schedules: Mutex::new(tier_schedules),
        }
    }

    /// Get the latest agent decision for a symbol
    pub async fn get_latest_decision(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Option<AgentDecision>, MarketIntelligenceError> {
        debug!("Querying latest decision for symbol: {}", symbol);

        // TODO: Query agent_decisions table from Model_Data database
        // For now, return None - will be implemented when database schema is added
        Ok(None)
    }

    /// Run the agent scheduler
    pub async fn run_scheduler(&self, cancellation: CancellationToken) -> anyhow::Result<()> {
        let config = self.config.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Service not properly initialized - missing config")
        })?;

        info!("Starting market intelligence agent scheduler");
        info!(
            "Monitoring symbols: {}",
            config.symbols.join(", ")
        );

        info!("Tier schedules:");
        info!(
            "  - HOT:    {} minutes (high volatility, active trading)",
            config.tier_schedule.hot_interval_minutes
        );
        info!(
            "  - WARM:   {} minutes (medium activity)",
            config.tier_schedule.warm_interval_minutes
        );
        info!(
            "  - STABLE: {} minutes (stable prices)",
            config.tier_schedule.stable_interval_minutes
        );
        info!(
            "  - COLD:   {} minutes (monitoring only)",
            config.tier_schedule.cold_interval_minutes
        );

        // Initialize symbol tier classification (placeholder)
        let symbol_tiers = self.classify_symbols(&config.symbols);
        info!(
            "Classified {} symbols into tiers",
            symbol_tiers.len()
        );

        // Main scheduler loop
        let mut tick_interval = tokio::time::interval(std::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = cancellation.cancelled() => {
                    info!("Scheduler cancellation signal received");
                    break;
                }
                _ = tick_interval.tick() => {
                    let now = Utc::now();

                    // Collect tiers that need to run
                    let tiers_to_run: Vec<SymbolTier> = {
                        let schedules = self.tier_schedules.lock().await;
                        schedules
                            .iter()
                            .filter_map(|(tier, schedule)| {
                                let should_run = match schedule.last_run {
                                    None => true,
                                    Some(last) => {
                                        let elapsed = (now - last).num_seconds() as u64;
                                        elapsed >= schedule.interval_secs
                                    }
                                };
                                if should_run { Some(*tier) } else { None }
                            })
                            .collect()
                    };

                    // Run agents for tiers that need it
                    for tier in tiers_to_run {
                        match self.run_tier_agents(tier, &symbol_tiers).await {
                            Ok(_) => {
                                let mut schedules = self.tier_schedules.lock().await;
                                if let Some(sched) = schedules.get_mut(&tier) {
                                    sched.last_run = Some(now);
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Error running {} tier agents: {}",
                                    tier.name(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        info!("Scheduler stopped");
        Ok(())
    }

    /// Classify symbols into tiers based on trading activity
    fn classify_symbols(&self, symbols: &[String]) -> HashMap<String, SymbolTier> {
        let mut classification = HashMap::new();

        // Placeholder classification: distribute evenly across tiers
        for (idx, symbol) in symbols.iter().enumerate() {
            let tier = match idx % 4 {
                0 => SymbolTier::Hot,
                1 => SymbolTier::Warm,
                2 => SymbolTier::Stable,
                _ => SymbolTier::Cold,
            };
            classification.insert(symbol.clone(), tier);
        }

        classification
    }

    /// Run agent analysis for symbols in a specific tier
    async fn run_tier_agents(
        &self,
        tier: SymbolTier,
        symbol_tiers: &HashMap<String, SymbolTier>,
    ) -> anyhow::Result<()> {
        let symbols: Vec<_> = symbol_tiers
            .iter()
            .filter(|(_, t)| *t == &tier)
            .map(|(s, _)| s.clone())
            .collect();

        if symbols.is_empty() {
            return Ok(());
        }

        info!(
            "Running agent analysis for {} tier ({} symbols)",
            tier.name(),
            symbols.len()
        );

        // For each symbol in the tier, run agent analysis
        for symbol in symbols {
            match self.analyze_symbol(&symbol).await {
                Ok(decision) => {
                    info!(
                        "{}: Decision={}, Confidence={:.2}%",
                        symbol,
                        decision.action,
                        decision.confidence * 100.0
                    );
                }
                Err(e) => {
                    warn!("Error analyzing {}: {}", symbol, e);
                }
            }
        }

        Ok(())
    }

    /// Analyze a single symbol and generate agent decision
    async fn analyze_symbol(&self, symbol: &str) -> anyhow::Result<AgentDecision> {
        let llm = self
            .llm
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?;

        // TODO: Gather sentiment data from data-ingestion module via ClientHub
        // For now, ask LLM for a generic market analysis

        let prompt = format!(
            "Analyze the market for {} crypto. \
            Provide a JSON response with: \
            {{\"action\": \"BUY\"|\"SELL\"|\"HOLD\", \
            \"confidence\": 0.0-1.0, \
            \"reasoning\": \"brief explanation\"}}",
            symbol
        );

        match llm.generate_response(&prompt).await {
            Ok(_response) => {
                // TODO: Parse JSON response from LLM
                // For now, return a placeholder decision
                Ok(AgentDecision {
                    symbol: symbol.to_string(),
                    action: "HOLD".to_string(),
                    confidence: 0.65,
                    reasoning: "Placeholder decision - LLM placeholder".to_string(),
                    created_at: Utc::now().naive_utc(),
                })
            }
            Err(e) => {
                error!("LLM error for {}: {}", symbol, e);
                Err(anyhow::anyhow!("LLM analysis failed: {}", e))
            }
        }
    }
}
