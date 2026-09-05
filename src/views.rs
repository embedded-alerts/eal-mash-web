use chrono::{DateTime, Utc};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use uuid::Uuid;

use crate::{
    config::AppEnvironment,
    models::{
        ApiHealth, MatchCandidate, MatchEvidence, PageIndexRecord, ScanReport, ScoreComponents,
        SearchResult, SemanticSearchResponse, SourceDomain,
    },
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum NoticeKind {
    Success,
    Error,
    Info,
}

impl NoticeKind {
    const fn class(self) -> &'static str {
        match self {
            Self::Success => "notice notice--success",
            Self::Error => "notice notice--error",
            Self::Info => "notice notice--info",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Notice {
    kind: NoticeKind,
    title: String,
    message: String,
}

impl Notice {
    pub(crate) fn new(
        kind: NoticeKind,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            message: message.into(),
        }
    }
}

pub(crate) fn layout(environment: AppEnvironment, tenant_id: Uuid) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="dark";
                title { "Embedded Alerts · Index Operations" }
                script src="https://unpkg.com/htmx.org@2.0.4" defer {}
                style { (PreEscaped(STYLES)) }
            }
            body {
                header class="topbar" {
                    a class="wordmark" href="/" aria-label="Embedded Alerts operations home" {
                        span class="mark" { "EA" }
                        span {
                            strong { "Embedded Alerts" }
                            small { "index operations" }
                        }
                    }
                    div class="runtime" {
                        span class="runtime__dot" {}
                        span { (environment.as_str()) }
                        code { (short_id(tenant_id)) }
                    }
                }

                main class="shell" {
                    section class="hero" {
                        div {
                            p class="eyebrow" { "DOMAIN-SCOPED SEMANTIC MONITORING" }
                            h1 { "Own the match. Borrow discovery." }
                            p class="hero__copy" {
                                "External indexes, feeds, and sitemaps may suggest URLs. Embedded Alerts only matches locally fetched, policy-approved, content-addressed revisions."
                            }
                        }
                        aside class="decision-card" {
                            p class="decision-card__label" { "Index authority" }
                            strong { "Embedded Alerts" }
                            dl {
                                div { dt { "Discovery" } dd { "hybrid" } }
                                div { dt { "Fetch" } dd { "bounded" } }
                                div { dt { "Delivery" } dd { "disabled" } }
                            }
                        }
                    }

                    section class="architecture-strip" aria-label="Indexing pipeline" {
                        (pipeline_step("01", "Discover", "allowlisted candidates"))
                        (pipeline_step("02", "Fetch", "robots + SSRF controls"))
                        (pipeline_step("03", "Normalize", "canonical URL + SHA-256"))
                        (pipeline_step("04", "Embed", "model-versioned segments"))
                        (pipeline_step("05", "Match", "explainable candidates"))
                    }

                    section class="panel" aria-labelledby="runtime-heading" {
                        div class="panel__head" {
                            div {
                                p class="section-index" { "00 / RUNTIME" }
                                h2 id="runtime-heading" { "Execution boundary" }
                            }
                            button class="quiet-button" hx-get="/partials/overview" hx-target="#overview" hx-swap="innerHTML" { "Refresh" }
                        }
                        div id="overview" hx-get="/partials/overview" hx-trigger="load" hx-swap="innerHTML" {
                            (loading("Reading API boundary…"))
                        }
                    }

                    section class="panel" aria-labelledby="sources-heading" {
                        div class="panel__head" {
                            div {
                                p class="section-index" { "01 / SOURCES" }
                                h2 id="sources-heading" { "Register a public source" }
                            }
                            p class="panel__hint" { "Exact DNS domains only. No arbitrary URL fetch route." }
                        }
                        div class="source-layout" {
                            form class="policy-form" hx-post="/partials/sources" hx-target="#sources-panel" hx-swap="innerHTML" hx-indicator="#source-submit-indicator" {
                                label {
                                    span { "Source name" }
                                    input type="text" name="name" maxlength="120" placeholder="Rust release notes" required;
                                }
                                label {
                                    span { "Domain" }
                                    input type="text" name="domain" maxlength="2048" placeholder="blog.rust-lang.org" required;
                                    small { "The API normalizes this to an HTTPS base URL unless a scheme is supplied." }
                                }
                                label {
                                    span { "Seed URLs" }
                                    textarea name="seed_urls" rows="3" placeholder="https://blog.rust-lang.org/\nhttps://blog.rust-lang.org/feed.xml" {}
                                    small { "Optional; one per line or comma-separated. Maximum 50." }
                                }
                                div class="form-row" {
                                    label {
                                        span { "Page budget" }
                                        input type="number" name="max_pages_per_scan" min="1" max="100" value="25" required;
                                    }
                                    label {
                                        span { "Source priority" }
                                        input type="number" name="source_priority" min="0" max="1" step="0.05" value="0.50" required;
                                    }
                                }
                                fieldset {
                                    legend { "Discovery adapters" }
                                    div class="check-grid" {
                                        (checkbox("mode_seed", "Seed URLs", true))
                                        (checkbox("mode_robots_sitemap", "robots.txt sitemaps", true))
                                        (checkbox("mode_sitemap", "Conventional sitemap", true))
                                        (checkbox("mode_rss", "RSS / Atom", false))
                                        (checkbox("mode_link_crawl", "Same-domain links", false))
                                        (checkbox("mode_external_index", "External index candidates", false))
                                    }
                                }
                                div class="policy-guards" {
                                    label class="check-line" {
                                        input type="checkbox" name="respect_robots" checked required;
                                        span { "Require robots.txt compliance" }
                                    }
                                    label class="check-line" {
                                        input type="checkbox" name="include_subdomains";
                                        span { "Include subdomains" }
                                    }
                                    label class="check-line" {
                                        input type="checkbox" name="enabled" checked;
                                        span { "Enable source" }
                                    }
                                }
                                button class="primary-button" type="submit" {
                                    span { "Register bounded source" }
                                    span id="source-submit-indicator" class="htmx-indicator" { "validating…" }
                                }
                            }
                            div id="sources-panel" hx-get="/partials/sources" hx-trigger="load" hx-swap="innerHTML" {
                                (loading("Loading source policies…"))
                            }
                        }
                    }

                    section class="panel" aria-labelledby="pages-heading" {
                        div class="panel__head" {
                            div {
                                p class="section-index" { "02 / REVISIONS" }
                                h2 id="pages-heading" { "Indexed page revisions" }
                            }
                            button class="quiet-button" hx-get="/partials/pages" hx-target="#pages-panel" hx-swap="innerHTML" { "Refresh" }
                        }
                        div id="pages-panel" hx-get="/partials/pages" hx-trigger="load" hx-swap="innerHTML" {
                            (loading("Loading immutable revisions…"))
                        }
                    }

                    section class="panel search-panel" aria-labelledby="search-heading" {
                        div class="panel__head" {
                            div {
                                p class="section-index" { "03 / MATCH LAB" }
                                h2 id="search-heading" { "Test semantic intent" }
                            }
                            span class="badge badge--safe" { "candidate-only" }
                        }
                        form class="search-form" hx-post="/partials/search" hx-target="#search-results" hx-swap="innerHTML" hx-indicator="#search-indicator" {
                            label class="search-form__query" {
                                span { "Natural-language interest" }
                                textarea name="query_text" rows="3" maxlength="2000" placeholder="Notify me when a Rust release changes async runtime behavior or supply-chain security guidance." required {}
                            }
                            div class="form-row form-row--three" {
                                label {
                                    span { "Threshold" }
                                    input type="number" name="threshold" min="0" max="1" step="0.01" value="0.72" required;
                                }
                                label {
                                    span { "Result limit" }
                                    input type="number" name="limit" min="1" max="100" value="20" required;
                                }
                                label {
                                    span { "Source UUID" }
                                    input type="text" name="source_id" placeholder="optional";
                                }
                            }
                            details class="candidate-gate" {
                                summary { "Create deduplicated candidates for an immutable alert-rule revision" }
                                p { "Both values are required together. This still does not send email, Slack, webhook, or other provider traffic." }
                                div class="form-row" {
                                    label {
                                        span { "Alert-rule UUID" }
                                        input type="text" name="alert_rule_id" placeholder="optional";
                                    }
                                    label {
                                        span { "Revision" }
                                        input type="number" name="alert_rule_revision" min="1" placeholder="optional";
                                    }
                                }
                            }
                            button class="primary-button" type="submit" {
                                span { "Run explainable match" }
                                span id="search-indicator" class="htmx-indicator" { "embedding…" }
                            }
                        }
                        div id="search-results" class="result-zone" {
                            (empty_state("No query run yet", "Results will show semantic, lexical, entity, recency, and source-priority evidence."))
                        }
                    }

                    section class="panel" aria-labelledby="matches-heading" {
                        div class="panel__head" {
                            div {
                                p class="section-index" { "04 / CANDIDATES" }
                                h2 id="matches-heading" { "Candidate review queue" }
                            }
                            button class="quiet-button" hx-get="/partials/matches" hx-target="#matches-panel" hx-swap="innerHTML" { "Refresh" }
                        }
                        div class="delivery-lock" {
                            strong { "Delivery lock engaged." }
                            span { "Cooldowns, grouping, provider idempotency, receipts, retries, and dead letters remain owned by DEN-3460." }
                        }
                        div id="matches-panel" hx-get="/partials/matches" hx-trigger="load" hx-swap="innerHTML" {
                            (loading("Loading candidate matches…"))
                        }
                    }
                }

                footer {
                    span { "Embedded Alerts · Rust / Axum / Maud / HTMX" }
                    span { "production fail-closed" }
                }
            }
        }
    }
}

pub(crate) fn overview_panel(health: &ApiHealth) -> Markup {
    html! {
        div class="status-grid" {
            (status_cell("API", &health.service, &health.status))
            (status_cell("Storage", &health.storage_mode, if health.database_connected { "database connected" } else { "process-local" }))
            (status_cell("Index", &health.semantic_index_mode, &health.embedding_mode))
            (status_cell("Model", &health.embedding_model, if health.supabase_configured { "Supabase configured" } else { "Supabase absent" }))
        }
        div class="boundary-note" {
            span class="boundary-note__mark" { "!" }
            div {
                strong { "API environment: " (health.environment) }
                p {
                    @if health.production_ready {
                        "The API reports production-ready. The Mash console still remains blocked until its own Shared Auth integration is certified."
                    } @else {
                        "The API correctly reports production_ready=false. State and tenant identity still require the DEN-3459 durability and Shared Auth gates."
                    }
                }
            }
        }
    }
}

pub(crate) fn sources_panel(sources: &[SourceDomain], maybe_notice: Option<&Notice>) -> Markup {
    html! {
        @if let Some(notice_value) = maybe_notice {
            (notice(notice_value))
        }
        div class="subhead" {
            strong { (sources.len()) " registered source" @if sources.len() != 1 { "s" } }
            span { "Scan buttons operate only on server-issued source UUIDs." }
        }
        @if sources.is_empty() {
            (empty_state("No sources registered", "Add an exact public domain to establish the index boundary."))
        } @else {
            div class="source-list" {
                @for source in sources {
                    (source_card(source))
                }
            }
        }
    }
}

fn source_card(source: &SourceDomain) -> Markup {
    let scan_target = format!("#scan-output-{}", source.id);
    let scan_id = format!("scan-output-{}", source.id);
    html! {
        article class="source-card" {
            div class="source-card__top" {
                div {
                    div class="source-title-row" {
                        h3 { (source.name) }
                        @if source.enabled {
                            span class="badge badge--active" { "enabled" }
                        } @else {
                            span class="badge" { "paused" }
                        }
                    }
                    a href=(source.base_url) target="_blank" rel="noopener noreferrer" { (source.host) }
                }
                div class="priority" {
                    span { "priority" }
                    strong { (percent(source.source_priority)) }
                }
            }
            dl class="source-stats" {
                div { dt { "Budget" } dd { (source.max_pages_per_scan) " pages" } }
                div { dt { "Subdomains" } dd { (yes_no(source.include_subdomains)) } }
                div { dt { "Robots" } dd { (if source.respect_robots { "required" } else { "unsafe" }) } }
                div { dt { "Tenant" } dd { code { (short_id(source.tenant_id)) } } }
            }
            div class="chips" {
                @for mode in &source.discovery_modes {
                    span { (mode.label()) }
                }
            }
            details {
                summary { "Policy details" }
                dl class="detail-list" {
                    div { dt { "Source UUID" } dd { code { (source.id) } } }
                    div { dt { "Base URL" } dd { code { (source.base_url) } } }
                    div { dt { "Created" } dd { (timestamp(source.created_at)) } }
                    div { dt { "Updated" } dd { (timestamp(source.updated_at)) } }
                    div {
                        dt { "Seed URLs" }
                        dd {
                            @if source.seed_urls.is_empty() {
                                "API default"
                            } @else {
                                ul class="compact-list" {
                                    @for seed in &source.seed_urls { li { code { (seed) } } }
                                }
                            }
                        }
                    }
                }
            }
            form hx-post=(format!("/partials/sources/{}/scan", source.id)) hx-target=(scan_target) hx-swap="innerHTML" hx-indicator=(format!("#scan-indicator-{}", source.id)) {
                button class="scan-button" type="submit" disabled[!source.enabled] {
                    span { "Run bounded scan" }
                    span id=(format!("scan-indicator-{}", source.id)) class="htmx-indicator" { "fetching…" }
                }
            }
            div id=(scan_id) class="scan-output" {}
        }
    }
}

pub(crate) fn scan_report(report: &ScanReport) -> Markup {
    html! {
        div class="scan-report" {
            div class="scan-report__head" {
                strong { "Scan complete" }
                code { (short_id(report.source_id)) }
            }
            div class="metric-row" {
                (metric("discovered", report.discovered_urls))
                (metric("attempted", report.attempted))
                (metric("created", report.created))
                (metric("updated", report.updated))
                (metric("unchanged", report.unchanged))
                (metric("robots blocked", report.rejected_by_robots))
                (metric("failed", report.failed))
            }
            p class="microcopy" {
                (report.sitemap_count) " sitemap resource(s) · extractor " code { (report.extractor_version) }
                " · model " code { (compact_json(&report.embedding_model)) }
            }
            @if !report.failures.is_empty() {
                details open {
                    summary { (report.failures.len()) " bounded fetch failure(s)" }
                    ul class="failure-list" {
                        @for failure in &report.failures {
                            li {
                                strong { (failure.code) }
                                span { (failure.message) }
                                code { (failure.url) }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn pages_panel(pages: &[PageIndexRecord]) -> Markup {
    html! {
        div class="subhead" {
            strong { (pages.len()) " indexed revision" @if pages.len() != 1 { "s" } }
            span { "Unchanged normalized content is a no-op; changed content links to its predecessor." }
        }
        @if pages.is_empty() {
            (empty_state("No page revisions", "Run a registered source scan before testing semantic matches."))
        } @else {
            div class="revision-list" {
                @for page in pages {
                    article class="revision-card" {
                        div class="revision-card__head" {
                            div {
                                p class="kicker" { "REVISION " code { (short_id(page.id)) } }
                                h3 {
                                    a href=(page.canonical_url) target="_blank" rel="noopener noreferrer" {
                                        (page.title.as_deref().unwrap_or(&page.canonical_url))
                                    }
                                }
                            }
                            span class="badge" { (page.segment_count) " segments" }
                        }
                        p class="summary" { (page.summary) }
                        div class="chips chips--keywords" {
                            @for keyword in page.keywords.iter().take(10) { span { (keyword) } }
                        }
                        @if !page.entities.is_empty() {
                            p class="entity-line" { strong { "Entities " } (page.entities.join(" · ")) }
                        }
                        dl class="detail-list detail-list--grid" {
                            div { dt { "Source" } dd { code { (short_id(page.source_id)) } } }
                            div { dt { "Fetched" } dd { (timestamp(page.fetched_at)) } }
                            div { dt { "Content hash" } dd { code { (short_hash(&page.content_hash)) } } }
                            div {
                                dt { "Predecessor" }
                                dd {
                                    @if let Some(previous) = page.previous_revision_id {
                                        code { (short_id(previous)) }
                                    } @else {
                                        "first revision"
                                    }
                                }
                            }
                            div { dt { "Extractor" } dd { code { (page.extractor_version) } } }
                            div { dt { "Model" } dd { code { (compact_json(&page.model)) } } }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn search_results(response: &SemanticSearchResponse) -> Markup {
    html! {
        div class="search-summary" {
            div {
                span { "Compared" }
                strong { (response.compared_pages) }
            }
            div {
                span { "Cross-model skipped" }
                strong { (response.skipped_cross_model_pages) }
            }
            div {
                span { "Candidates created" }
                strong { (response.candidate_matches_created) }
            }
            div {
                span { "Returned" }
                strong { (response.results.len()) }
            }
        }
        p class="microcopy" {
            "Query: “" (response.query_text) "” · model " code { (compact_json(&response.model)) }
            @if let Some(cursor) = &response.next_cursor {
                " · next cursor " code { (cursor) }
            }
        }
        @if response.results.is_empty() {
            (empty_state("No result crossed the threshold", "Lower the threshold carefully or ingest additional policy-approved revisions."))
        } @else {
            div class="result-list" {
                @for result in &response.results {
                    (search_result_card(result))
                }
            }
        }
    }
}

fn search_result_card(result: &SearchResult) -> Markup {
    html! {
        article class="result-card" {
            div class="result-card__score" {
                strong { (percent(result.score)) }
                span { "combined" }
            }
            div class="result-card__body" {
                p class="kicker" { "PAGE " code { (short_id(result.page_revision_id)) } " · SOURCE " code { (short_id(result.source_id)) } }
                h3 {
                    a href=(result.canonical_url) target="_blank" rel="noopener noreferrer" {
                        (result.title.as_deref().unwrap_or(&result.canonical_url))
                    }
                }
                p class="summary" { (result.summary) }
                (score_breakdown(&result.components))
                (evidence_list(&result.evidence))
                dl class="detail-list detail-list--grid" {
                    div { dt { "Fetched" } dd { (timestamp(result.fetched_at)) } }
                    div { dt { "Content hash" } dd { code { (short_hash(&result.content_hash)) } } }
                    div { dt { "Model" } dd { code { (compact_json(&result.model)) } } }
                }
            }
        }
    }
}

pub(crate) fn matches_panel(matches: &[MatchCandidate]) -> Markup {
    html! {
        div class="subhead" {
            strong { (matches.len()) " candidate" @if matches.len() != 1 { "s" } }
            span { "Candidates are evidence records, not delivery attempts." }
        }
        @if matches.is_empty() {
            (empty_state("No candidates", "Run a semantic query with both alert-rule UUID and revision to create deduplicated candidates."))
        } @else {
            div class="candidate-list" {
                @for candidate in matches {
                    article class="candidate-card" {
                        div class="candidate-card__head" {
                            div {
                                p class="kicker" { "MATCH " code { (short_id(candidate.id)) } }
                                h3 {
                                    a href=(candidate.canonical_url) target="_blank" rel="noopener noreferrer" { (candidate.canonical_url) }
                                }
                            }
                            div class="candidate-score" {
                                strong { (percent(candidate.score)) }
                                span class="badge badge--safe" { (candidate.state) }
                            }
                        }
                        (score_breakdown(&candidate.components))
                        (evidence_list(&candidate.evidence))
                        details {
                            summary { "Immutable identity and provenance" }
                            dl class="detail-list" {
                                div { dt { "Tenant" } dd { code { (candidate.tenant_id) } } }
                                div { dt { "Rule" } dd { code { (candidate.alert_rule_id) } " @ revision " (candidate.alert_rule_revision) } }
                                div { dt { "Page revision" } dd { code { (candidate.page_revision_id) } } }
                                div { dt { "Source" } dd { code { (candidate.source_id) } } }
                                div { dt { "Match key" } dd { code { (candidate.match_key) } } }
                                div { dt { "Content hash" } dd { code { (candidate.content_hash) } } }
                                div { dt { "Query hash" } dd { code { (candidate.query_hash) } } }
                                div { dt { "Model" } dd { code { (compact_json(&candidate.model)) } } }
                                div { dt { "Created" } dd { (timestamp(candidate.created_at)) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn notice(value: &Notice) -> Markup {
    html! {
        div class=(value.kind.class()) role="status" {
            strong { (value.title) }
            p { (value.message) }
        }
    }
}

fn score_breakdown(components: &ScoreComponents) -> Markup {
    html! {
        div class="score-grid" title=(format!("weights: {}", compact_json(&components.weights))) {
            (score_component("semantic", components.semantic))
            (score_component("lexical", components.lexical))
            (score_component("entity", components.entity))
            (score_component("recency", components.recency))
            (score_component("source", components.source_priority))
        }
    }
}

fn evidence_list(evidence: &[MatchEvidence]) -> Markup {
    html! {
        @if !evidence.is_empty() {
            details class="evidence" {
                summary { "Top segment evidence" }
                ol {
                    @for item in evidence {
                        li {
                            div {
                                strong { (item.page_segment_kind) " ↔ " (item.query_segment_kind) }
                                span { (percent(item.weighted_similarity)) " weighted · " (percent(item.similarity)) " raw" }
                            }
                            p { (item.page_text) }
                        }
                    }
                }
            }
        }
    }
}

fn pipeline_step(index: &str, title: &str, detail: &str) -> Markup {
    html! {
        div class="pipeline-step" {
            span { (index) }
            strong { (title) }
            small { (detail) }
        }
    }
}

fn checkbox(name: &str, label: &str, selected: bool) -> Markup {
    html! {
        label class="check-line" {
            input type="checkbox" name=(name) checked[selected];
            span { (label) }
        }
    }
}

fn status_cell(label: &str, value: &str, detail: &str) -> Markup {
    html! {
        div class="status-cell" {
            span { (label) }
            strong { (value) }
            small { (detail) }
        }
    }
}

fn metric(label: &str, value: usize) -> Markup {
    html! {
        div {
            strong { (value) }
            span { (label) }
        }
    }
}

fn score_component(label: &str, value: f32) -> Markup {
    html! {
        div {
            span { (label) }
            strong { (percent(value)) }
            meter min="0" max="1" value=(format!("{value:.4}")) { (percent(value)) }
        }
    }
}

fn loading(message: &str) -> Markup {
    html! {
        div class="loading" role="status" {
            span class="loading__bar" {}
            span { (message) }
        }
    }
}

fn empty_state(title: &str, detail: &str) -> Markup {
    html! {
        div class="empty-state" {
            strong { (title) }
            p { (detail) }
        }
    }
}

fn timestamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M UTC").to_string()
}

fn percent(value: f32) -> String {
    format!("{:.1}%", value.clamp(0.0, 1.0) * 100.0)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn short_id(value: Uuid) -> String {
    value.to_string().chars().take(8).collect()
}

fn short_hash(value: &str) -> String {
    if value.chars().count() <= 16 {
        value.to_owned()
    } else {
        format!("{}…{}", &value[..8], &value[value.len() - 8..])
    }
}

fn compact_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "unknown".into())
}

const STYLES: &str = r#"
:root {
  --paper: #11110f;
  --paper-2: #181816;
  --paper-3: #22221f;
  --ink: #f4f0e5;
  --muted: #aaa796;
  --line: #3a3932;
  --signal: #ffb000;
  --signal-soft: #4a3510;
  --good: #98d89b;
  --bad: #ff8f7f;
  --cyan: #8ac9d1;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: var(--ink);
  background: var(--paper);
}
* { box-sizing: border-box; }
body { margin: 0; min-width: 320px; background: var(--paper); color: var(--ink); }
a { color: inherit; text-decoration-color: var(--signal); text-underline-offset: .2em; }
code { font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace; overflow-wrap: anywhere; }
button, input, textarea { font: inherit; }
button { cursor: pointer; }
.topbar { position: sticky; top: 0; z-index: 20; display: flex; align-items: center; justify-content: space-between; padding: .85rem clamp(1rem, 4vw, 4rem); border-bottom: 1px solid var(--line); background: rgba(17,17,15,.96); backdrop-filter: blur(12px); }
.wordmark { display: inline-flex; align-items: center; gap: .8rem; text-decoration: none; }
.wordmark .mark { display: grid; place-items: center; width: 2.25rem; height: 2.25rem; border: 1px solid var(--signal); color: var(--signal); font: 700 .72rem/1 monospace; letter-spacing: .08em; }
.wordmark strong, .wordmark small { display: block; }
.wordmark small { color: var(--muted); font-size: .72rem; letter-spacing: .12em; text-transform: uppercase; }
.runtime { display: flex; align-items: center; gap: .55rem; color: var(--muted); font-size: .78rem; text-transform: uppercase; letter-spacing: .08em; }
.runtime__dot { width: .5rem; height: .5rem; border-radius: 50%; background: var(--signal); box-shadow: 0 0 0 .25rem var(--signal-soft); }
.shell { width: min(1380px, calc(100% - 2rem)); margin: 0 auto; padding: 4rem 0 7rem; }
.hero { display: grid; grid-template-columns: minmax(0, 1fr) minmax(240px, 360px); gap: clamp(2rem, 7vw, 8rem); align-items: end; padding: 2rem 0 3.5rem; }
.eyebrow, .section-index, .kicker { margin: 0 0 .65rem; color: var(--signal); font: 650 .72rem/1.4 monospace; letter-spacing: .14em; text-transform: uppercase; }
h1, h2, h3, p { margin-top: 0; }
h1 { max-width: 900px; margin-bottom: 1.15rem; font-size: clamp(3rem, 8vw, 7.2rem); line-height: .88; letter-spacing: -.065em; font-weight: 650; }
h2 { margin-bottom: 0; font-size: clamp(1.65rem, 3vw, 2.75rem); letter-spacing: -.04em; }
h3 { margin-bottom: .4rem; font-size: 1.15rem; }
.hero__copy { max-width: 780px; color: var(--muted); font-size: clamp(1.05rem, 1.8vw, 1.35rem); line-height: 1.55; }
.decision-card { border-left: 3px solid var(--signal); padding: 1.25rem 0 1.25rem 1.5rem; }
.decision-card__label { color: var(--muted); text-transform: uppercase; letter-spacing: .12em; font-size: .7rem; }
.decision-card > strong { display: block; margin-bottom: 1.25rem; font-size: 1.65rem; }
.decision-card dl { display: grid; gap: .5rem; margin: 0; }
.decision-card dl div { display: flex; justify-content: space-between; gap: 1rem; border-top: 1px solid var(--line); padding-top: .5rem; }
dt { color: var(--muted); }
dd { margin: 0; }
.architecture-strip { display: grid; grid-template-columns: repeat(5, 1fr); border: 1px solid var(--line); margin-bottom: 2rem; }
.pipeline-step { min-height: 120px; padding: 1rem; border-right: 1px solid var(--line); display: flex; flex-direction: column; justify-content: space-between; }
.pipeline-step:last-child { border-right: 0; }
.pipeline-step > span { color: var(--signal); font: .72rem monospace; }
.pipeline-step strong { font-size: 1.05rem; }
.pipeline-step small { color: var(--muted); }
.panel { border-top: 1px solid var(--line); padding: 2.2rem 0 3.4rem; }
.panel__head { display: flex; justify-content: space-between; align-items: end; gap: 2rem; margin-bottom: 1.75rem; }
.panel__hint, .subhead span, .microcopy { color: var(--muted); font-size: .82rem; }
.quiet-button, .scan-button, .primary-button { border: 1px solid var(--line); background: transparent; color: var(--ink); padding: .7rem 1rem; }
.quiet-button:hover, .scan-button:hover { border-color: var(--signal); }
.primary-button { display: inline-flex; justify-content: center; gap: .7rem; border-color: var(--signal); background: var(--signal); color: #18130a; font-weight: 700; }
.primary-button:hover { background: #ffc13d; }
.scan-button:disabled { opacity: .45; cursor: not-allowed; }
.htmx-indicator { display: none; }
.htmx-request .htmx-indicator, .htmx-request.htmx-indicator { display: inline; }
.source-layout { display: grid; grid-template-columns: minmax(300px, .85fr) minmax(0, 1.6fr); gap: clamp(2rem, 5vw, 5rem); align-items: start; }
.policy-form, .search-form { display: grid; gap: 1rem; padding: 1.25rem; border: 1px solid var(--line); background: var(--paper-2); }
label > span, legend { display: block; margin-bottom: .4rem; color: var(--muted); font-size: .76rem; letter-spacing: .06em; text-transform: uppercase; }
input, textarea { width: 100%; border: 1px solid var(--line); border-radius: 0; background: var(--paper); color: var(--ink); padding: .78rem .85rem; }
input:focus, textarea:focus { outline: 2px solid var(--signal-soft); border-color: var(--signal); }
textarea { resize: vertical; line-height: 1.45; }
label small { display: block; margin-top: .35rem; color: var(--muted); line-height: 1.45; }
.form-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .8rem; }
.form-row--three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
fieldset { margin: 0; border: 1px solid var(--line); padding: .9rem; }
.check-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .55rem; }
.check-line { display: flex; align-items: center; gap: .55rem; color: var(--ink); }
.check-line input { width: auto; accent-color: var(--signal); }
.check-line span { margin: 0; color: var(--ink); font-size: .82rem; letter-spacing: 0; text-transform: none; }
.policy-guards { display: grid; gap: .55rem; }
.subhead { display: flex; justify-content: space-between; gap: 1rem; align-items: baseline; margin-bottom: 1rem; }
.source-list, .revision-list, .candidate-list, .result-list { display: grid; gap: 1rem; }
.source-card, .revision-card, .candidate-card, .result-card { border: 1px solid var(--line); background: var(--paper-2); padding: 1rem; }
.source-card__top, .revision-card__head, .candidate-card__head { display: flex; justify-content: space-between; gap: 1rem; align-items: start; }
.source-title-row { display: flex; align-items: center; gap: .65rem; }
.priority { text-align: right; }
.priority span { display: block; color: var(--muted); font-size: .68rem; text-transform: uppercase; }
.priority strong { color: var(--signal); }
.badge { display: inline-flex; align-items: center; min-height: 1.55rem; padding: .15rem .5rem; border: 1px solid var(--line); color: var(--muted); font: .68rem monospace; text-transform: uppercase; letter-spacing: .08em; }
.badge--active, .badge--safe { border-color: #466a49; color: var(--good); }
.source-stats, .detail-list { display: grid; gap: .45rem; margin: 1rem 0; }
.source-stats { grid-template-columns: repeat(4, minmax(0, 1fr)); border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); padding: .75rem 0; }
.source-stats dt, .detail-list dt { font-size: .68rem; text-transform: uppercase; letter-spacing: .08em; }
.source-stats dd { margin-top: .2rem; }
.detail-list > div { display: grid; grid-template-columns: minmax(100px, .32fr) minmax(0, 1fr); gap: .7rem; }
.detail-list--grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.detail-list--grid > div { display: block; }
.chips { display: flex; flex-wrap: wrap; gap: .4rem; margin: .8rem 0; }
.chips span { border: 1px solid var(--line); padding: .2rem .45rem; color: var(--cyan); font-size: .72rem; }
.chips--keywords span { color: var(--muted); }
details { margin: .85rem 0; }
summary { cursor: pointer; color: var(--muted); }
.compact-list { margin: 0; padding-left: 1rem; }
.scan-output:not(:empty) { margin-top: 1rem; }
.scan-report { border-left: 3px solid var(--good); background: #151b15; padding: 1rem; }
.scan-report__head { display: flex; justify-content: space-between; gap: 1rem; }
.metric-row { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: .5rem; margin: .8rem 0; }
.metric-row div { border-top: 1px solid #3a4b3b; padding-top: .4rem; }
.metric-row strong, .metric-row span { display: block; }
.metric-row span { color: var(--muted); font-size: .67rem; }
.failure-list { display: grid; gap: .65rem; padding-left: 1.2rem; }
.failure-list span, .failure-list code { display: block; }
.failure-list code { color: var(--muted); font-size: .74rem; }
.summary { color: #d1cdbc; line-height: 1.55; }
.entity-line { color: var(--muted); font-size: .8rem; }
.search-panel { background: linear-gradient(90deg, transparent 0, transparent 2%, rgba(255,176,0,.025) 2%, rgba(255,176,0,.025) 98%, transparent 98%); }
.search-form { grid-template-columns: 1fr; max-width: 1000px; }
.candidate-gate { border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); padding: .75rem 0; }
.candidate-gate p { color: var(--muted); font-size: .82rem; }
.result-zone { margin-top: 1.25rem; }
.search-summary { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); border: 1px solid var(--line); margin-bottom: 1rem; }
.search-summary div { padding: .8rem; border-right: 1px solid var(--line); }
.search-summary div:last-child { border-right: 0; }
.search-summary span, .search-summary strong { display: block; }
.search-summary span { color: var(--muted); font-size: .7rem; text-transform: uppercase; }
.search-summary strong { margin-top: .2rem; font-size: 1.45rem; }
.result-card { display: grid; grid-template-columns: 110px minmax(0, 1fr); gap: 1.25rem; }
.result-card__score { border-right: 1px solid var(--line); }
.result-card__score strong { display: block; color: var(--signal); font-size: 1.7rem; }
.result-card__score span { color: var(--muted); font-size: .7rem; text-transform: uppercase; }
.score-grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: .5rem; margin: .85rem 0; }
.score-grid > div { border-top: 1px solid var(--line); padding-top: .4rem; }
.score-grid span, .score-grid strong { display: block; }
.score-grid span { color: var(--muted); font-size: .67rem; text-transform: uppercase; }
meter { width: 100%; height: .35rem; accent-color: var(--signal); }
.evidence ol { display: grid; gap: .65rem; padding-left: 1.2rem; }
.evidence li > div { display: flex; justify-content: space-between; gap: 1rem; }
.evidence li span { color: var(--muted); font-size: .72rem; }
.evidence li p { margin: .25rem 0 0; color: #d1cdbc; font-size: .82rem; line-height: 1.45; }
.delivery-lock { display: flex; gap: .7rem; padding: .8rem 1rem; border-left: 3px solid var(--bad); background: #241614; margin-bottom: 1rem; }
.delivery-lock span { color: #d5a39b; }
.candidate-score { display: flex; align-items: center; gap: .7rem; }
.candidate-score strong { color: var(--signal); font-size: 1.4rem; }
.notice { border-left: 3px solid var(--cyan); background: #132124; padding: .8rem 1rem; margin-bottom: 1rem; }
.notice p { margin: .25rem 0 0; color: var(--muted); }
.notice--success { border-color: var(--good); background: #151f15; }
.notice--error { border-color: var(--bad); background: #251614; }
.notice--info { border-color: var(--cyan); }
.loading { display: flex; align-items: center; gap: .75rem; min-height: 72px; color: var(--muted); }
.loading__bar { width: 2.5rem; height: 2px; background: var(--signal); animation: pulse 1.1s infinite alternate ease-in-out; }
@keyframes pulse { to { transform: scaleX(.25); opacity: .35; } }
.empty-state { border: 1px dashed var(--line); padding: 1.25rem; color: var(--muted); }
.empty-state strong { color: var(--ink); }
.empty-state p { margin: .35rem 0 0; }
footer { display: flex; justify-content: space-between; gap: 1rem; padding: 1.2rem clamp(1rem, 4vw, 4rem); border-top: 1px solid var(--line); color: var(--muted); font: .7rem monospace; text-transform: uppercase; }
@media (max-width: 980px) {
  .hero, .source-layout { grid-template-columns: 1fr; }
  .architecture-strip { grid-template-columns: repeat(2, 1fr); }
  .pipeline-step { border-bottom: 1px solid var(--line); }
  .source-stats, .metric-row, .score-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .form-row--three { grid-template-columns: 1fr; }
}
@media (max-width: 640px) {
  .shell { width: min(100% - 1.1rem, 1380px); padding-top: 2rem; }
  .topbar { padding-inline: .75rem; }
  .runtime code { display: none; }
  .architecture-strip, .form-row, .check-grid, .detail-list--grid, .search-summary { grid-template-columns: 1fr; }
  .pipeline-step, .search-summary div { border-right: 0; }
  .result-card { grid-template-columns: 1fr; }
  .result-card__score { border-right: 0; border-bottom: 1px solid var(--line); padding-bottom: .6rem; }
  .panel__head, .subhead, .source-card__top, .candidate-card__head { align-items: start; flex-direction: column; }
  footer { flex-direction: column; }
}
"#;
