# Solana Trading Engine

A safety-first Solana market-monitoring and algorithmic trading system.

## Development stages

1. Live market-data ingestion
2. Token/event normalization
3. Security and liquidity filters
4. Strategy signals
5. Paper trading
6. Backtesting and performance measurement
7. Transaction simulation
8. Controlled live execution

**Live trading is disabled during initial development.** Never commit seed phrases, private keys, or wallet secrets.

## Architecture

- `rust-engine/` — latency-sensitive ingestion, analysis, strategy, risk, and execution components
- `python-control/` — analytics, backtesting, reporting, and control-plane tooling
- `dashboard/` — web dashboard
- `database/` — PostgreSQL/Supabase schema and migrations
- `configs/` — environment-specific configuration
- `tests/` — automated tests

## Security

Secrets must be supplied through environment variables or a secrets manager. Public wallet addresses are safe to display; private keys and seed phrases are never required in source control or chat.
