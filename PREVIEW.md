# Trading Engine Preview

The repository includes a browser control-center preview at the root index.html.

It is intentionally paper-mode: it does not invent live market data, expose RPC keys, sign transactions, or broadcast transactions.

The Rust worker remains the critical path. The browser/Vercel UI is the control plane and preview, not the always-on trading worker.

Production gates remain: dedicated database, redundant private RPC, validated Raydium decoding, paper/backtest validation, transaction simulation, confirmation/retry/blockhash expiry, external key management, and explicit server-side live enablement.