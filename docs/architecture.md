# Mash console architecture

## Responsibility boundary

`eal-mash-web` is a server-rendered operator client for `eal-api`. The browser never receives the development tenant selector and never calls the API directly. All source registration, scans, page reads, semantic searches, and candidate reads flow through the server-side `ApiClient`.

The console does not duplicate source-domain validation or semantic scoring. Local form validation exists only for fast feedback and safer defaults; the API remains authoritative.

The operation-level choice among constrained database reads, stateless HTTP,
stateful API connections, and asynchronous messaging is recorded in
[`web-api-data-access.md`](web-api-data-access.md). The current console uses only
the stateless HTTP path.

## Hybrid discovery

External indexes are useful for recall, not authority. A provider result may enter the crawl queue only as a URL candidate. The ingestion runtime must then apply the registered exact-host/path policy, per-hop DNS and redirect checks, robots policy, rate and byte budgets, readable-text extraction, canonical URL identity, normalized SHA-256 content identity, and model-versioned embedding generation.

## Realtime and delivery

The console uses explicit HTMX refreshes instead of the current process-local WebSocket because tenant-filtered authenticated events are not certified. Candidate creation is optional and requires an immutable alert-rule UUID/revision pair. No console route can send a notification.

## Production gates

The service rejects production startup while it depends on `x-eal-tenant-id`. Shared Auth, durable repositories, explicit origins/CSP, self-hosted frontend assets, and the DEN-3460 outbox/delivery state machine are mandatory release gates.
