# Embedded Alerts operator console

This is the canonical Mash/Maud + Axum + HTMX operator surface for domain-scoped semantic indexing.

It intentionally does less than the crawler and delivery services:

- It registers and lists tenant-scoped source policies through `eal-api`.
- It runs natural-language semantic searches through the model-versioned server-side query route.
- It creates durable match candidates for review.
- It does **not** accept arbitrary fetch URLs, lease crawl jobs, fetch pages, generate page embeddings, or send notifications.

## Architecture boundary

1. `eal-interfaces` owns source, page-revision, embedding-space, search, and candidate contracts.
2. `eal-sync` owns discovery and bounded public-page ingestion. Search engines, feeds, and sitemaps may propose URLs, but every URL must still pass the registered source policy, robots rules, canonicalization, public-network checks, redirect re-validation, bounded fetch, and revision hashing.
3. `eal-api` owns tenant authorization, PostgreSQL/pgvector persistence, model-versioned semantic search, and candidate creation.
4. DEN-3460 owns dedupe, cooldowns, an outbox, retries, receipts, and delivery. The crawler never sends.

The API remains the contract authority. Source creation uses a JSON editor so this console does not duplicate or silently drift from `CreateSourcePolicy`. Server-owned `tenant_id`, `id`, and timestamp fields are rejected locally before the request crosses the API boundary.

## Run locally

```bash
cp .env.example .env
set -a
. ./.env
set +a
cargo run
```

`EAL_API_BASE_URL` may use plain HTTP only for loopback development. The console blocks production startup until Shared Auth session propagation has been certified; a development tenant UUID is mandatory in the meantime.

## Required validation

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
python3 scripts/verify_contract.py
```

## Deliberately disabled

The dashboard shows crawler and notification controls as unavailable. They must not be enabled until the durable queue-leasing ingestion loop, PostgreSQL alert-rule repository, Shared Auth boundary, and DEN-3460 outbox/delivery canaries are all green.
