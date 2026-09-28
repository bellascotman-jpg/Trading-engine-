# Solana Trading Engine — Production Architecture

## Mission
A professional Solana market-observation, strategy, paper-trading, and eventually controlled execution system. The system is designed to make decisions from verifiable on-chain/market data and never fabricate metrics.

## Operating modes
- `observe`: ingest and analyze only; no orders.
- `paper`: generate simulated entries/exits and calculate realized/unrealized P&L.
- `simulation`: construct and simulate transactions without broadcasting.
- `live`: explicitly enabled by the owner after all risk gates pass.

Live execution is disabled by default and requires multiple independent configuration gates.

## Architecture

### Rust critical path
- Pump.fun/event ingestion
- Solana RPC/WebSocket and optional Yellowstone gRPC ingestion
- Raydium/pool discovery adapters
- event normalization
- token/account decoding
- authority checks
- holder concentration analysis
- buy/sell flow aggregation
- strategy evaluation
- position state machine
- transaction construction/simulation
- priority-fee policy
- Jito submission/status adapters
- confirmations, retries, blockhash expiry handling
- circuit breakers and risk limits

### Control plane
- dashboard/API
- configuration
- historical analytics
- backtesting
- reports
- Telegram notifications
- AI-assisted research/analysis (never authoritative for execution safety)

### Persistence
PostgreSQL/Supabase tables for tokens, pools, events, trades, holders, risk assessments, signals, paper positions, orders, transactions, executions, P&L, engine health, strategy configurations and audit logs.

## Decision model
Every candidate receives explicit evidence and a deterministic decision:
- `Pass`
- `Reject`
- `Watch`

Required security data must be present. Missing security data is a rejection, not permission to trade.

## Initial strategy inputs
- mint authority state
- freeze authority state
- liquidity
- holder concentration / top-10 concentration
- buy/sell transaction flow
- buy/sell SOL flow
- unique buyers/sellers
- short-window volume
- price velocity
- token age
- expected price impact
- exit-liquidity estimate

The initial user strategy parameters are configurable rather than hard-coded: buy/sell pressure threshold, take-profit, stop-loss, trailing drawdown, maximum position size, maximum open positions and daily loss limit.

## Execution principles
Solana priority fees increase scheduling priority; they do not guarantee inclusion. The execution layer must estimate compute requirements, set appropriate limits/prices, track blockhash validity, simulate during development, and monitor confirmation. Jito bundles are an optional execution route and are not treated as guaranteed landing or guaranteed protection from every ordering risk.

## Safety
- No seed phrases or private keys in source control, chat, logs, database, or dashboard.
- Secrets only through deployment secret stores/environment variables.
- No live trading in development/test builds.
- Global kill switch.
- Per-token exposure cap.
- Per-position exposure cap.
- Maximum open positions.
- Daily loss circuit breaker.
- Maximum slippage / price-impact guard.
- Stale-data rejection.
- Duplicate-order/idempotency protection.
- Transaction simulation gate where applicable.
- Blockhash expiry handling.
- Emergency exit path.
- Immutable audit events for decisions and execution attempts.

## Development order
1. Repository foundation and CI
2. Rust event model and configuration
3. Solana/Pump.fun ingestion
4. Persistent event storage
5. Security/risk engine
6. Signal engine
7. Paper-trading engine
8. Backtesting and metrics
9. Transaction builder + simulation
10. Jito/RPC execution adapters
11. Position/exits engine
12. Dashboard and alerts
13. Production observability
14. Controlled live activation

No claim of profitability is made by the software. Strategy performance must be demonstrated empirically through historical and paper-trading data before live activation.
