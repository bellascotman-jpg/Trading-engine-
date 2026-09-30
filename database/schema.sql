-- Solana Trading Engine production schema
-- Apply to the dedicated Supabase project. Engine writes use the server-side secret only.
create extension if not exists pgcrypto;

create table if not exists public.tokens (id uuid primary key default gen_random_uuid(), mint text not null unique, symbol text, name text, decimals int, created_at timestamptz not null default now(), first_seen_at timestamptz not null default now(), migrated_at timestamptz, metadata jsonb not null default '{}'::jsonb);
create table if not exists public.market_events (id uuid primary key default gen_random_uuid(), event_id text not null unique, mint text references public.tokens(mint), source text not null, kind text not null, slot bigint, signature text, observed_at timestamptz not null default now(), sol_amount numeric, token_amount numeric, is_buy boolean, trader text, raw_event jsonb not null default '{}'::jsonb);
create table if not exists public.pools (id uuid primary key default gen_random_uuid(), mint text references public.tokens(mint), dex text not null, pool_address text not null unique, base_mint text, quote_mint text, liquidity_usd numeric, price_usd numeric, volume_5m_usd numeric, volume_1h_usd numeric, created_at timestamptz, last_seen_at timestamptz not null default now(), metadata jsonb not null default '{}'::jsonb);
create table if not exists public.security_snapshots (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), checked_at timestamptz not null default now(), mint_authority_revoked boolean, freeze_authority_revoked boolean, top_10_non_bonding_pct numeric, holder_count bigint, liquidity_usd numeric, risk_state text not null, reasons jsonb not null default '[]'::jsonb, evidence jsonb not null default '[]'::jsonb);
create table if not exists public.token_audits (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), generated_at timestamptz not null default now(), verdict text not null, audit_score numeric not null, rug_confidence numeric not null, honeypot_confidence numeric not null, scam_confidence numeric not null, reasons jsonb not null default '[]'::jsonb, evidence jsonb not null default '[]'::jsonb, model_version text not null default 'evidence-v1');
create table if not exists public.market_snapshots (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), observed_at timestamptz not null default now(), price_usd numeric, fdv_usd numeric, liquidity_usd numeric, volume_5m_usd numeric, volume_1h_usd numeric, buys_5m bigint, sells_5m bigint, dex text, pool_address text);
create table if not exists public.market_candles (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), interval text not null, bucket_start timestamptz not null, open numeric, high numeric, low numeric, close numeric, volume_usd numeric, buy_volume_usd numeric, sell_volume_usd numeric, trades bigint, unique_buyers bigint, unique_sellers bigint, unique(mint,interval,bucket_start));
create table if not exists public.strategy_decisions (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), created_at timestamptz not null default now(), decision text not null, score numeric, buy_sol_1m numeric, sell_sol_1m numeric, unique_buyers_1m bigint, unique_sellers_1m bigint, evidence jsonb not null default '{}'::jsonb);
create table if not exists public.scenario_forecasts (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), generated_at timestamptz not null default now(), multiple numeric not null, probability numeric not null, horizon text not null, model_version text not null default 'evidence-v1', basis jsonb not null default '[]'::jsonb);
create table if not exists public.paper_positions (id uuid primary key default gen_random_uuid(), mint text not null references public.tokens(mint), opened_at timestamptz not null default now(), closed_at timestamptz, entry_price numeric not null, exit_price numeric, quantity numeric not null, invested_sol numeric not null, realized_pnl_sol numeric, realized_roi_pct numeric, exit_reason text, status text not null default 'open', peak_price numeric);
create table if not exists public.orders (id uuid primary key default gen_random_uuid(), mint text references public.tokens(mint), position_id uuid, mode text not null, side text not null, quantity numeric not null, max_slippage_bps bigint not null, priority_fee_lamports bigint, jito_tip_lamports bigint, status text not null default 'created', created_at timestamptz not null default now(), submitted_at timestamptz, confirmed_at timestamptz, failure_reason text);
create table if not exists public.executions (id uuid primary key default gen_random_uuid(), order_id uuid not null references public.orders(id), signature text, bundle_id text, slot bigint, landed boolean not null default false, confirmation_latency_ms bigint, fee_lamports bigint, priority_fee_lamports bigint, jito_tip_lamports bigint, created_at timestamptz not null default now(), raw_result jsonb not null default '{}'::jsonb);
create table if not exists public.risk_events (id uuid primary key default gen_random_uuid(), created_at timestamptz not null default now(), severity text not null, event_type text not null, mint text, message text not null, metadata jsonb not null default '{}'::jsonb);
create table if not exists public.engine_health (id uuid primary key default gen_random_uuid(), component text not null, observed_at timestamptz not null default now(), mode text not null, rpc_ok boolean not null, pumpportal_ok boolean not null, current_slot bigint, rpc_latency_ms bigint, websocket_latency_ms bigint, open_positions bigint not null default 0, daily_pnl_sol numeric not null default 0, last_error text);

alter table public.tokens enable row level security;
alter table public.market_events enable row level security;
alter table public.pools enable row level security;
alter table public.security_snapshots enable row level security;
alter table public.token_audits enable row level security;
alter table public.market_snapshots enable row level security;
alter table public.market_candles enable row level security;
alter table public.strategy_decisions enable row level security;
alter table public.scenario_forecasts enable row level security;
alter table public.paper_positions enable row level security;
alter table public.orders enable row level security;
alter table public.executions enable row level security;
alter table public.risk_events enable row level security;
alter table public.engine_health enable row level security;

-- Public dashboard access is intentionally not granted here. Server-side API routes use SUPABASE_SECRET_KEY.
