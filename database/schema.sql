-- Production schema matching the connected Supabase trading project.
create extension if not exists pgcrypto;
create table if not exists public.tokens(mint text primary key,name text,symbol text,token_program text,decimals integer,total_supply numeric,creator text,created_at_chain timestamptz,first_seen_at timestamptz not null default now(),last_seen_at timestamptz,status text,metadata jsonb default '{}'::jsonb);
create table if not exists public.token_security(id uuid primary key default gen_random_uuid(),mint text not null,observed_at timestamptz not null default now(),mint_authority_revoked boolean,freeze_authority_revoked boolean,metadata_mutable boolean,transfer_hook_detected boolean,permanent_delegate_detected boolean,transfer_fee_bps integer,non_transferable boolean,security_status text,rug_risk_score numeric,honeypot_risk_score numeric,scam_risk_score numeric,evidence jsonb default '[]'::jsonb);
create table if not exists public.holders(id uuid primary key default gen_random_uuid(),mint text not null,owner text not null,balance numeric,pct_supply numeric,is_creator boolean default false,is_lp boolean default false,is_burn_address boolean default false,cluster_id text,observed_at timestamptz not null default now());
create table if not exists public.liquidity_pools(id uuid primary key default gen_random_uuid(),mint text not null,dex text not null,pool_address text not null,quote_mint text,base_reserve numeric,quote_reserve numeric,liquidity_usd numeric,price_usd numeric,price_impact_1k_bps numeric,observed_at timestamptz not null default now(),unique(mint,dex,pool_address));
create table if not exists public.market_trades(id uuid primary key default gen_random_uuid(),mint text,signature text,slot bigint,trader text,side text,price_usd numeric,amount_tokens numeric,amount_sol numeric,amount_usd numeric,observed_at timestamptz not null default now());
create table if not exists public.market_candles(id uuid primary key default gen_random_uuid(),mint text,timeframe text,bucket_start timestamptz,open numeric,high numeric,low numeric,close numeric,volume_usd numeric,buy_volume_usd numeric,sell_volume_usd numeric,buy_count integer,sell_count integer,unique_buyers integer,unique_sellers integer,vwap numeric,unique(mint,timeframe,bucket_start));
create table if not exists public.audit_reports(id uuid primary key default gen_random_uuid(),mint text,generated_at timestamptz not null default now(),audit_version text,verdict text,rug_confidence numeric,honeypot_confidence numeric,scam_confidence numeric,liquidity_confidence numeric,holder_confidence numeric,market_structure_confidence numeric,momentum_confidence numeric,data_quality_confidence numeric,critical_flags jsonb,warnings jsonb,evidence jsonb,model_version text,raw jsonb);
create table if not exists public.scenario_forecasts(id uuid primary key default gen_random_uuid(),mint text,audit_id uuid,generated_at timestamptz not null default now(),horizon text,multiple numeric,target_price_usd numeric,target_market_cap_usd numeric,probability numeric,confidence_low numeric,confidence_high numeric,model_version text,feature_snapshot jsonb);
create table if not exists public.paper_positions(id uuid primary key default gen_random_uuid(),mint text,status text,opened_at timestamptz not null default now(),closed_at timestamptz,entry_price_usd numeric,exit_price_usd numeric,quantity_tokens numeric,remaining_tokens numeric,realized_pnl_usd numeric,realized_roi_pct numeric,max_favorable_excursion_pct numeric,max_adverse_excursion_pct numeric,exit_reason text,metadata jsonb);
create table if not exists public.paper_orders(id uuid primary key default gen_random_uuid(),position_id uuid,mint text,side text,order_type text,requested_price_usd numeric,filled_price_usd numeric,quantity_tokens numeric,slippage_bps numeric,simulated_fee_usd numeric,status text,created_at timestamptz not null default now());
create table if not exists public.risk_events(id uuid primary key default gen_random_uuid(),mint text,severity text,event_type text,message text,evidence jsonb,created_at timestamptz not null default now());
create table if not exists public.engine_health(component text primary key,status text,last_heartbeat timestamptz,latency_ms numeric,details jsonb);

alter table public.tokens enable row level security;
alter table public.token_security enable row level security;
alter table public.holders enable row level security;
alter table public.liquidity_pools enable row level security;
alter table public.market_trades enable row level security;
alter table public.market_candles enable row level security;
alter table public.audit_reports enable row level security;
alter table public.scenario_forecasts enable row level security;
alter table public.paper_positions enable row level security;
alter table public.paper_orders enable row level security;
alter table public.risk_events enable row level security;
alter table public.engine_health enable row level security;

-- The engine writes with SUPABASE_SECRET_KEY/server-side credentials. No service-role/secret key is exposed to the browser.
