# HAI3 Frontend Architecture — CricoAI v2

## Definition of Done

- [ ] `p2` - **ID**: `cpt-cf-cricoai-dod-frontend-complete`

All 6 screensets render with correct data, auth flow works end-to-end, OData queries work from frontend, theme switching works, mock mode works without backend.

## 1. Overview

The CricoAI v2 frontend is a HAI3 application (React 19 + TypeScript) consuming the cyberfabric-core REST API and SSE streams. It replaces the existing React dashboard and two Streamlit dashboards with a single app using HAI3's screenset architecture.

## 2. App Setup

```typescript
// src/app/cricoai.ts
import { createHAI3 } from '@hai3/react';

export const app = createHAI3()
  .use(full())              // Full layout (header, menu, sidebar, screen)
  .use(authPlugin())        // JWT auth with cyberfabric-core
  .use(themePlugin())       // Dark/light/dracula themes
  .use(i18nPlugin())        // English + Romanian
  .build();
```

## 3. Screensets

Each screenset is a vertical slice with its own screens, menu items, API services, translations, and state.

| Screenset | Screens | API Service |
|-----------|---------|-------------|
| **TradingScreenset** | TradingOverview, OpenOrders, TradeHistory, PairEvolution, AccountBalances, AssetValueChart | TradingApiService |
| **ModelsScreenset** | ModelSummary, TrainingHistory, CalibrationMetrics, RegimeAnalysis, PredictionHistory | ModelsApiService |
| **AgentsScreenset** | AgentOverview, RecentDecisions, SentimentTrends, LlmUsage, TradeImpact, AgentEffectiveness | AgentsApiService |
| **PerformanceScreenset** | PerformanceOverview, StrategyComparison, PerformanceAlerts | PerformanceApiService |
| **SettingsScreenset** | TradingConfig, PairManagement, SignalViewer, SystemControl | ConfigApiService |
| **AdminScreenset** | UserManagement, RoleManagement, TwoFactorSetup, IpBlocking, AuditLogs | AuthApiService |

## 4. API Services

API services use HAI3's `BaseApiService` with `RestProtocol`. Tenant context flows from the JWT token — no `env` query parameter.

```typescript
// src/app/api/TradingApiService.ts
import { BaseApiService, RestProtocol } from '@hai3/api';

export class TradingApiService extends BaseApiService {
  static serviceId = 'trading' as const;

  async getStatsSummary(timeRange: string) {
    return this.protocol(RestProtocol)
      .get<TradingStatsSummary>('/trading-dashboard/v1/stats/summary', {
        params: { time_range: timeRange }
      });
  }

  async getRecentTrades(params: ODataParams) {
    return this.protocol(RestProtocol)
      .get<PageResponse<Trade>>('/trading-dashboard/v1/stats/recent-trades', {
        params: {
          $filter: params.filter,
          $orderby: params.orderby,
          $select: params.select,
          limit: params.limit,
          cursor: params.cursor,
        }
      });
  }

  async getOpenOrders() {
    return this.protocol(RestProtocol)
      .get<PageResponse<OpenOrder>>('/trading-dashboard/v1/orders/open');
  }
}
```

### OData Query Support

All list endpoints support DNA-compliant OData queries. The frontend passes `$filter`, `$orderby`, `$select`, `limit`, and `cursor` as query parameters.

```typescript
// src/app/types/odata.ts
export interface ODataParams {
  filter?: string;    // e.g., "symbol eq 'BTCUSDT' and created_at gt 2025-01-01T00:00:00.000Z"
  orderby?: string;   // e.g., "created_at desc"
  select?: string;    // e.g., "id,symbol,pnl"
  limit?: number;     // default 25, max 200
  cursor?: string;    // opaque cursor from page_info
}

export interface PageResponse<T> {
  items: T[];
  page_info: {
    limit: number;
    next_cursor: string | null;
    prev_cursor: string | null;
  };
}
```

### Error Handling

All API errors follow RFC 9457 Problem Details:

```typescript
// src/app/types/errors.ts
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  trace_id?: string;
  errors?: Array<{ field: string; code: string; message: string }>;
}
```

### Write Operations

Write endpoints include DNA concurrency headers:

```typescript
// src/app/api/ConfigApiService.ts
async updateProfitTarget(symbol: string, profitPercent: number, etag: string) {
  return this.protocol(RestProtocol)
    .put(`/config-manager/v1/pairs/${symbol}/profit-target`, {
      body: { profit_percent: profitPercent },
      headers: {
        'If-Match': etag,
        'Idempotency-Key': crypto.randomUUID(),
      }
    });
}
```

### Mock API Support

Each API service registers mock plugins for development without backend:

```typescript
export class TradingApiService extends BaseApiService {
  constructor() {
    super();
    this.registerMockPlugin(new TradingMockPlugin());
  }
}
```

## 5. State Management

HAI3's event-driven pattern with eventBus + Redux slices:

```typescript
// src/app/events/trading.events.ts
declare module '@hai3/state' {
  interface EventPayloadMap {
    'trading/stats/fetch': { timeRange: string };
    'trading/stats/loaded': TradingStatsSummary;
    'trading/orders/fetch': void;
    'trading/orders/loaded': PageResponse<OpenOrder>;
  }
}

// src/app/actions/trading.actions.ts
export const fetchTradingStats = (timeRange: string) => {
  eventBus.emit('trading/stats/fetch', { timeRange });
};

// src/app/effects/trading.effects.ts
eventBus.on('trading/stats/fetch', async ({ timeRange }) => {
  const api = apiRegistry.get(TradingApiService);
  const stats = await api.getStatsSummary(timeRange);
  store.dispatch(setTradingStats(stats));
});
```

## 6. SSE Integration

Subscribe to real-time event streams from cyberfabric-core modules:

```typescript
// src/app/effects/sse.effects.ts
const priceSource = new EventSource('/market-data/v1/events/prices');
priceSource.onmessage = (event) => {
  const price = JSON.parse(event.data);
  store.dispatch(updatePrice(price));
};

const tradeSource = new EventSource('/trading-core/v1/events/trades');
tradeSource.onmessage = (event) => {
  const trade = JSON.parse(event.data);
  store.dispatch(addTradeEvent(trade));
};

const agentSource = new EventSource('/market-intelligence/v1/events/decisions');
agentSource.onmessage = (event) => {
  const decision = JSON.parse(event.data);
  store.dispatch(addAgentDecision(decision));
};
```

## 7. UI Components

Use `@hai3/uikit` components (shadcn/ui + Radix UI):

- **Data display:** `Card`, `Table`, `Chart`
- **Controls:** `Button`, `Input`, `Select`, `Switch`
- **Modals:** `Dialog`, `AlertDialog`
- **Navigation:** `Tabs`, `Badge`, `Tooltip`
- **Theme-aware** via CSS custom properties

## 8. Traceability

- **PRD**: [PRD.md](./PRD.md) — `cpt-cricoai-actor-frontend`
- **DESIGN**: [DESIGN.md](./DESIGN.md) — system topology, SSE integration
- **DECOMPOSITION**: [DECOMPOSITION.md](./DECOMPOSITION.md) — Phase 4
