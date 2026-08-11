# eal-mash-web

**Embedded Alerts operator console — Rust + Axum + Maud + HTMX**

This service is the operational surface for domain-scoped public-page indexing and explainable semantic matching. It intentionally does not implement crawler policy, vector scoring, tenant authorization, persistence, or notification delivery itself. Those boundaries remain in `eal-api`, `eal-sync`, `eal-interfaces`, and `eal-libs`.

## Indexing decision

Embedded Alerts owns the authoritative match index. External search providers, RSS/Atom feeds, sitemaps, and manual seeds may propose candidate URLs, but a candidate is not a match. Every page must be fetched through a registered source policy, revalidated through redirects and DNS, checked against robots rules and resource budgets, canonicalized, content-addressed, embedded with explicit model provenance, and scored locally.

The console exposes these operator workflows:

- register exact public domains with bounded discovery adapters and page budgets;
- run scans only for server-issued source UUIDs;
- inspect immutable page revisions, content hashes, model provenance, keywords, entities, and embedding segment counts;
- test natural-language interests with explainable semantic, lexical, entity, recency, and source-priority scores;
- optionally create deduplicated candidates by supplying an immutable alert-rule UUID/revision pair;
- review candidates while notification delivery remains locked.

There is no arbitrary-URL fetch endpoint and no delivery action in this service.

## Development

```bash
cp .env.example .env
cargo run
```

Required configuration:

| Variable | Purpose |
| --- | --- |
| `APP_ENV` | `development` or `test`; `production` intentionally fails closed |
| `EAL_API_BASE_URL` | Fixed `http`/`https` base URL for `eal-api`; URL credentials are rejected |
| `EAL_TENANT_ID` | Explicit development tenant UUID sent server-side as `x-eal-tenant-id` |
| `HOST` / `PORT` | Console listener, default `0.0.0.0:8081` |

The API client blocks redirects so tenant context cannot be forwarded to an unexpected host, applies connection/request timeouts, and caps decoded API responses at 4 MiB.

## Production boundary

`APP_ENV=production|prod` fails startup. Production deployment requires all of the following:

1. Shared Auth claims replace the development tenant header.
2. `eal-api` sources, revisions, embeddings, rules, matches, and tenant-filtered events use durable PostgreSQL/pgvector repositories.
3. Explicit CORS/origin policy and tenant-isolation/restart canaries pass in `embedded-alerts-test`.
4. DEN-3460 delivers from a durable outbox with cooldowns, grouping, provider idempotency, receipts, retries, and dead letters.
5. HTMX is self-hosted with a reviewed CSP rather than loaded from a public CDN.

## Validation

```bash
python3 scripts/verify_repo.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Linear: DEN-3461; related DEN-3459, DEN-3460, and DEN-3462.
