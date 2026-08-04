# eal-mash-web

Maud + Axum + SeaORM + Supabase/PostgreSQL + HTMX + WebSocket web server for Embedded Alerts.

**Product:** Embedded Alerts — Embedding-based alerting for semantically relevant new information.

Define semantic alert rules, ingest source documents, compare embeddings, rank matches, and deliver explainable notifications.

## Safety and production boundary

Similarity scores are ranking signals, not truth guarantees. Production ingestion must respect source terms, robots rules, privacy requirements, retention limits, and notification consent.

This repository is an executable bootstrap, not a production deployment. Before live
use, add authentication, tenant authorization, rate limits, durable migrations,
observability, backups, incident response, dependency review, and secret management.
## Stack

Maud renders escaped server-side HTML, Axum serves HTTP/WebSockets, HTMX handles
progressive updates, SeaORM connects to Supabase-compatible PostgreSQL, and the
browser refreshes fragments after WebSocket notifications.

`DATABASE_URL` and `SUPABASE_URL` are optional for the in-memory bootstrap. Never
expose a Supabase service-role key to browser code.
