# Web/API data-access decision

Embedded Alerts adopts the
[portfolio four-path ADR](https://github.com/ORESoftware/k8s-cluster/blob/main/docs/architecture/web-api-data-access.md)
for [ORESoftware/k8s-cluster#1399](https://github.com/ORESoftware/k8s-cluster/issues/1399)
and [DEN-3960](https://linear.app/denman/issue/DEN-3960/document-4-web-server-to-api-server-data-access-patterns-across-10).
The selection is per operation and must be revisited before this scaffold is
promoted to production.

## Current boundary and production target

`eal-mash-web` currently combines Maud pages, HTTP handlers, an optional
database connection used only by health reporting, an in-memory `Vec<Item>`,
and same-process browser WebSocket wake-ups. It is a foundation scaffold: the
in-memory create route is not authoritative persistence, the database handle is
not a reviewed P1 reader, and the browser WebSocket is not P3.

The production split is:

| Operation | Path | Decision |
| --- | --- | --- |
| Render a tenant's alert-rule/read-model list | P1 only after hardening, otherwise P2 | P1 requires a distinct read-only role, forced tenant scope, and allow-listed views. |
| Create, change, pause, or delete an alert rule | P2: stateless HTTP | `eal-api` owns validation, idempotency, and the product write. |
| Receive low-latency API invalidation hints | P3 only if measured | One bounded authenticated subscription per web replica; P2 refresh remains authoritative. |
| Ingest sources, evaluate rules, and deliver notifications | P4: asynchronous queue | API-owned jobs use durable consumers, commit-before-ack, retry budgets, and DLQ handling. |
| Browser `/ws` connection | Browser/web transport | Same-process wake-up hint; not web-server-to-API P3 and never authoritative. |

## Path 1: constrained direct reads

P1 is not enabled by the current optional `DATABASE_URL`. Before direct reads,
provision a distinct web-read credential with no DML, DDL, ownership,
membership, or `BYPASSRLS`; expose only reviewed stable views; derive tenant and
actor scope from verified identity; force RLS or an equivalent predicate; and
prove cross-tenant denial. Bound the pool and query timeout, cancel work when
the HTTP request is canceled, and never fall back to a writer credential.

Replica/view reads may be stale. UI routes that require read-after-write or an
authoritative evaluation/delivery state use P2. Browser-session state, if added,
is web-owned and isolated from product-domain tables; it does not justify
product DML through P1.

## Path 2: stateless HTTP

P2 is the default for alert-rule reads and the required path for every product
mutation. `eal-api` verifies the caller, tenant, scope, schema version, quotas,
and request bounds. Mutation clients generate one stable idempotency key per
logical action and reuse it across transient retries. Set connect and total
deadlines, cap attempts with jitter, honor `Retry-After`, and do not retry
authentication, authorization, validation, or conflict responses.

Propagate W3C trace context and a request ID. Record route template, status
class, latency, timeout, retry count, idempotency replay/conflict outcome, and
bounded pool pressure. Do not log alert content, source documents, credentials,
tenant/customer identifiers, or raw request URLs. A saturated API rejects work
explicitly; the web tier does not create an unbounded queue or switch to P1.

## Path 3: bounded stateful API connection

P3 is reserved for low-latency invalidation/evaluation hints after a measured
need. The connection must authenticate the web workload and tenant
subscriptions, cap connections per replica, set connect/idle/lifetime
deadlines, heartbeat, bound inbound/outbound buffers, reconnect with capped
jitter, and drain during shutdown. Sequence gaps, overflow, or disconnect force
an authoritative P2 resync. No alert rule or delivery result is committed by a
P3 frame. The existing browser WebSocket remains a separate browser/web path.

## Path 4: asynchronous NATS or message queue

P4 is the target for source ingestion, rule evaluation, and delivery work whose
lifetime exceeds an HTTP request. Envelopes are versioned and contain tenant,
actor/service identity, trace context, stable message/idempotency ID, bounded
references, attempt metadata, and expiry—not source bodies, secrets, or complete
customer payloads. Consumers are durable, idempotent, concurrency-bounded, and
ack only after the result/outbox commit. Configure retry limits, backoff,
dead-letter policy, graceful drain, and queue age, redelivery, DLQ, and handler
latency metrics. Publication means accepted, not completed; P2 exposes status.

## Consistency, failure, and backpressure

- `eal-api` owns product writes and authoritative status. The MASH process owns
  only presentation and any future isolated browser-session state.
- P1 returns its snapshot; P2 returns accepted/committed API state; P3 is a hint;
  P4 is asynchronous acceptance until the API result changes.
- Database, API, stream, and broker failures fail closed with an explicit
  unavailable or pending state. They never widen tenant scope or change paths.
- Shutdown stops admission, drains bounded HTTP/P3 work, and leaves unacked P4
  messages eligible for redelivery.

## Contracts, schema, and migrations

Wire contracts belong in `eal-interfaces`; reusable domain behavior belongs in
`eal-libs`; `eal-api` owns the persistence adapter and product schema. This
scaffold has no reviewed production migration, which is a deployment blocker,
not permission to create tables at process startup. The future declarative
schema and migration history must be named in `eal-api`, verified for tenant
isolation, and applied by a one-shot migration identity never mounted into the
MASH web or request-serving API processes.
