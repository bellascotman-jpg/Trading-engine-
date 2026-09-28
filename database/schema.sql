-- Solana Trading Engine data model
-- Design migration only: apply to the dedicated trading Supabase project after project selection.
-- Security: every exposed table must use RLS; no service-role secrets belong in clients.

create extension if not exists pgcrypto;

create table if not exists public.tokens (
  id uuid primary key default gen_random_uuid(),
  mint text not null unique,
  symbol text,
  name text,
  decimals int,
  created_at timestamptz not null default now(),
  first_seen_at timestamptz not null default now(),
  migrated_at timestamptz,
  metadata jsonb not null default '{}'::jsonb
);

create table if not exists public.market_events (
  id uuid primary key default gen_random_uuid(),
  event_id text not null unique,
  mint text references public.tokens(mint),
  source text not null,
  kind text not null,
  slot bigint,
  signature text,
  observed_at timestamptz not null default now(),
  sol_amount numeric,
  token_amount numeric,
  is_buy boolean,
  raw_event jsonb not null default '{}'::jsonb
);

create table if not exists public.pools (
  id uuid primary key default gen_random_uuid(),
  mint text references public.tokens(mint),
  dex text not null,
  pool_address text not null unique,
  base_mint text,
  quote_mint text,
  liquidity_usd numeric,
  created_at timestamptz,
  last_seen_at timestamptz not null default now(),
  metadata jsonb not null default '{}'::jsonb
);

create table if not exists public.security_snapshots (
  id uuid primary key default gen_random_uuid(),
  mint text not null references public.tokens(mint),
  checked_at timestamptz not null default now(),
  mint_authority_revoked boolean,
  freeze_authority_revoked boolean,
  top_10_non_bonding_pct numeric,
  holder_count bigint,
  liquidity_usd numeric,
  risk_state text not null,
  reasons jsonb not null default '[]'::jsonb
);

create table if not exists public.strategy_decisions (
  id uuid primary key default gen_random_uuid(),
  mint text not null references public.tokens(mint),
  created_at timestamptz not null default now(),
  decision text not null,
  score numeric,
  buy_sol_1m numeric,
  sell_sol_1m numeric,
  unique_buyers_1m bigint,
  unique_sellers_1m bigint,
  evidence jsonb not null default '{}'::jsonb
);

create table if not exists public.paper_positions (
  id uuid primary key default gen_random_uuid(),
  mint text not null references public.tokens(mint),
  opened_at timestamptz not null default now(),
  closed_at timestamptz,
  entry_price numeric not null,
  exit_price numeric,
  quantity numeric not null,
  invested_sol numeric not null,
  realized_pnl_sol numeric,
  exit_reason text,
  status text not null default 'open'
);

create table if not exists public.orders (
  id uuid primary key default gen_random_uuid(),
  mint text references public.tokens(mint),
  position_id uuid,
  mode text not null,
  side text not null,
  quantity numeric not null,
  max_slippage_bps bigint not null,
  priority_fee_lamports bigint,
  status text not null default 'created',
  created_at timestamptz not null default now(),
  submitted_at timestamptz,
  confirmed_at timestamptz,
  failure_reason text
);

create table if not exists public.executions (
  id uuid primary key default gen_random_uuid(),
  order_id uuid not null references public.orders(id),
  signature text,
  slot bigint,
  landed boolean not null default false,
  confirmation_latency_ms bigint,
  fee_lamports bigint,
  priority_fee_lamports bigint,
  created_at timestamptz not null default now(),
  raw_result jsonb not null default '{}'::jsonb
);

create table if not exists public.risk_events (
  id uuid primary key default gen_random_uuid(),
  created_at timestamptz not null default now(),
  severity text not null,
  event_type text not null,
  mint text,
  message text not null,
  metadata jsonb not null default '{}'::jsonb
);

create table if not exists public.engine_health (
  id uuid primary key default gen_random_uuid(),
  observed_at timestamptz not null default now(),
  mode text not null,
  rpc_ok boolean not null,
  pumpportal_ok boolean not null,
  current_slot bigint,
  rpc_latency_ms bigint,
  websocket_latency_ms bigint,
  open_positions bigint not null default 0,
  daily_pnl_sol numeric not null default 0,
  last_error text
);

-- Defense in depth: tables are not publicly writable/readable merely because they exist.
alter table public.tokens enable row level security;
alter table public.market_events enable row level security;
alter table public.pools enable row level security;
alter table public.security_snapshots enable row level security;
alter table public.strategy_decisions enable row level security;
alter table public.paper_positions enable row level security;
alter table public.orders enable row level security;
alter table public.executions enable row level security;
alter table public.risk_events enable row level security;
alter table public.engine_health enable row level security;
