# Web/API data-access decision

Embedded Alerts adopts the
[portfolio four-path ADR](https://github.com/ORESoftware/k8s-cluster/blob/main/docs/architecture/web-api-data-access.md)
for [ORESoftware/k8s-cluster#1399](https://github.com/ORESoftware/k8s-cluster/issues/1399)
and [DEN-3960](https://linear.app/denman/issue/DEN-3960/document-4-web-server-to-api-server-data-access-patterns-across-10).
The path is selected per operation; one process must not inherit every data
authority merely because several transports exist.

## Current boundary

`eal-mash-web` is a server-rendered development/operator client. The browser
talks only to the web process, and the web process sends every source, scan,
page, search, and candidate operation through the server-side `ApiClient` to
`eal-api`. The current implementation therefore uses P2 exclusively:

| Operation | Current path | Decision |
| --- | --- | --- |
| Read API health, sources, pages, and match candidates | P2: stateless HTTP | `eal-api` remains authoritative; an API outage renders an explicit unavailable state. |
| Register a source, request a scan, or run a semantic search that may create candidates | P2: stateless HTTP | The API owns validation, tenant authorization, idempotency, persistence, and write effects. |
| Direct database read | Disabled | The web process has no database dependency or credential and must not add one without the P1 gates below. |
| Stateful web-to-API stream | Disabled | HTMX performs explicit refreshes; tenant-filtered authenticated events have not been certified. |
| NATS or queue publication | Disabled in the web process | Ingestion, evaluation, and delivery orchestration belong to API-owned workers and durable outbox consumers. |

The development tenant header is not production authentication. The service
already refuses production configuration while it depends on that header, and
this transport decision does not weaken that release gate.

## P1: constrained direct reads

P1 is intentionally disabled. If measurements later justify it, use a distinct
web-read role with no DML, DDL, ownership, role membership, migration authority,
or `BYPASSRLS`. Grant only reviewed stable views, derive tenant and actor scope
from verified Shared Auth identity, force RLS or equivalent predicates, and
test cross-tenant denial. Bound the pool and query timeout, cancel abandoned
work, expose replica staleness, and never fall back to a writer credential.

Reads that require authoritative evaluation/delivery state, read-after-write,
private source policy decisions, or authorization stay on P2. Browser-session
storage, if introduced, is isolated web-owned state and does not grant product
table access.

## P2: stateless HTTP

P2 is implemented by `ApiClient`. It uses one configured API base URL, blocks
redirects so tenant context cannot cross an authority boundary, applies bounded
connect and total deadlines, caps response bodies, and reduces untrusted error
responses to typed safe notices. The browser never receives the development
tenant selector and never calls the API directly.

Before production, replace `x-eal-tenant-id` with verified Shared Auth workload
and user context, constrain the API origin, propagate W3C trace context plus a
request ID, and attach one stable idempotency key to each logical mutation.
Reuse that key for bounded transient retries. Do not retry authentication,
authorization, validation, conflict, or policy decisions. Record route
template, status class, latency, timeout, retry count, idempotency outcome, and
bounded connection-pool pressure without logging tenant identifiers, source
content, query text, credentials, or raw URLs.

API saturation and outages return an explicit unavailable result. The web tier
must not build an unbounded retry queue, silently switch to P1, or report a
source, scan, search, candidate, or delivery action as committed when the API
did not confirm it.

## P3: bounded stateful API connection

P3 remains disabled until a measured latency need and a versioned subscription
contract exist. Any future connection authenticates the web workload and each
tenant subscription, caps connections per replica, sets connect, idle, and
lifetime deadlines, heartbeats, bounds both directions, reconnects with capped
jitter, and drains on shutdown. Sequence gaps, buffer overflow, and disconnect
force an authoritative P2 resync. Frames are invalidation hints only; they do
not register sources, schedule scans, create candidates, or complete delivery.

## P4: asynchronous NATS or message queue

P4 belongs behind the API for source ingestion, crawl/index work, embedding
generation, rule evaluation, and notification delivery. Versioned envelopes
carry tenant and service identity, trace context, stable message/idempotency ID,
bounded references, attempt metadata, and expiry—not source bodies, query text,
provider credentials, or complete customer payloads. Consumers are durable,
idempotent, concurrency-bounded, and acknowledge only after result/outbox
commit. Configure retry budgets, backoff, dead-letter policy, graceful drain,
and queue-age, redelivery, DLQ, and handler-latency metrics.

The web process may request work through P2 and poll authoritative status; it
does not publish directly to internal subjects. HTTP acceptance means queued or
accepted, not completed.

## Ownership, consistency, and migrations

- `eal-mash-web` owns presentation and any future isolated browser session
  state. It does not own product persistence, migrations, or delivery effects.
- `eal-api` owns source policy, product authorization, writes, and authoritative
  source, scan, page, match, and delivery status.
- P1 would return a documented snapshot; P2 returns API-confirmed state; P3 is
  a hint; P4 is asynchronous acceptance until the authoritative result changes.
- Database, API, stream, broker, and provider failures fail closed and never
  widen tenant scope or cause an implicit transport fallback.

Wire contracts belong in `eal-interfaces`; reusable domain behavior belongs in
the shared core libraries; the API-side persistence boundary owns declarative
schema and migrations. Migrations run under a one-shot identity that is never
mounted into either request-serving web or API processes.
