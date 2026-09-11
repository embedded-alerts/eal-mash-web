use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde_json::Value;

use crate::config::Environment;

const CSS: &str = r#"
:root{font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;color:#162018;background:#edf1e9;font-synthesis:none}*{box-sizing:border-box}body{margin:0;min-height:100vh;background:radial-gradient(circle at 15% 0%,#f8f4e8 0,transparent 34rem),linear-gradient(145deg,#edf1e9,#dde6dd 58%,#d4ded9)}a{color:inherit}.shell{width:min(1240px,calc(100% - 32px));margin:0 auto;padding:28px 0 64px}.masthead{display:flex;align-items:flex-end;justify-content:space-between;gap:24px;padding:22px 0 30px;border-bottom:1px solid #9aaa9d}.eyebrow{font-size:.72rem;letter-spacing:.16em;text-transform:uppercase;font-weight:800;color:#506255}.brand{margin:.25rem 0 0;font-family:Georgia,serif;font-size:clamp(2.1rem,6vw,4.9rem);line-height:.94;letter-spacing:-.045em}.env{display:inline-flex;gap:.55rem;align-items:center;border:1px solid #829087;border-radius:999px;padding:.55rem .8rem;background:#f7f8f1;font:700 .72rem/1 ui-monospace,SFMono-Regular,Menlo,monospace}.dot{width:.55rem;height:.55rem;border-radius:50%;background:#c08b3d;box-shadow:0 0 0 4px #c08b3d22}.status-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:1px;margin:26px 0;background:#9aaa9d;border:1px solid #9aaa9d}.metric{background:#f5f6ef;padding:18px;min-height:112px}.metric b{display:block;margin-top:15px;font-family:Georgia,serif;font-size:1.5rem;font-weight:500}.metric small{color:#5f6e63}.guardrail{display:grid;grid-template-columns:1.15fr 1fr;gap:0;border:1px solid #283b2f;background:#1d2e24;color:#f4f0df;margin:0 0 28px}.guardrail>div{padding:24px}.guardrail>div+div{border-left:1px solid #526056}.guardrail h2{font-family:Georgia,serif;font-size:1.65rem;margin:0 0 8px}.guardrail p{margin:0;color:#cad4c9;line-height:1.55}.guardrail ol{margin:0;padding-left:1.2rem;line-height:1.7;color:#e7e7d8}.workspace{display:grid;grid-template-columns:minmax(0,1.15fr) minmax(320px,.85fr);gap:22px;align-items:start}.stack{display:grid;gap:22px}.panel{border:1px solid #9aaa9d;background:#f8f8f2;box-shadow:0 12px 38px #22362a12}.panel-head{display:flex;align-items:flex-start;justify-content:space-between;gap:18px;padding:20px 22px;border-bottom:1px solid #bdc8bd}.panel-head h2{font:500 1.55rem/1.1 Georgia,serif;margin:0}.panel-head p{margin:.35rem 0 0;color:#667269;font-size:.9rem;line-height:1.45}.panel-body{padding:22px}.tag{display:inline-flex;align-items:center;border:1px solid #8f9e94;border-radius:999px;padding:.34rem .55rem;font:700 .67rem/1 ui-monospace,SFMono-Regular,Menlo,monospace;text-transform:uppercase;letter-spacing:.08em;background:#eef1e8}.tag.warn{background:#f5e9cf;border-color:#b99b5a}.source-list{display:grid;gap:10px}.source{border:1px solid #b8c3b9;background:white}.source summary{cursor:pointer;display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;padding:15px 16px;list-style:none}.source summary::-webkit-details-marker{display:none}.source strong{display:block}.source code{font-size:.72rem;color:#617066}.source pre,.json{margin:0;border-top:1px solid #d2d9d1;background:#17251d;color:#dfe8dc;padding:16px;max-height:420px;overflow:auto;font:12px/1.55 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;overflow-wrap:anywhere}.empty{padding:28px;border:1px dashed #9eaaa0;text-align:center;color:#667269;background:#f1f3eb}.form-grid{display:grid;grid-template-columns:1fr 1fr;gap:14px}.field{display:grid;gap:7px}.field.full{grid-column:1/-1}.field label{font-size:.72rem;font-weight:800;letter-spacing:.09em;text-transform:uppercase;color:#536157}.field input,.field textarea{width:100%;border:1px solid #9aaa9d;background:#fffef8;padding:.78rem .82rem;border-radius:0;color:#162018;font:inherit}.field textarea{min-height:180px;resize:vertical;font:12px/1.55 ui-monospace,SFMono-Regular,Menlo,monospace}.field input:focus,.field textarea:focus{outline:3px solid #cf9d4c42;border-color:#846d3d}.hint{font-size:.76rem;color:#6d786f;line-height:1.45}.actions{display:flex;align-items:center;gap:12px;margin-top:16px}.button{appearance:none;border:1px solid #21372a;background:#21372a;color:#fff;padding:.72rem 1rem;font:800 .75rem/1 ui-monospace,SFMono-Regular,Menlo,monospace;letter-spacing:.06em;text-transform:uppercase;cursor:pointer}.button:hover{background:#334d3c}.button.secondary{background:transparent;color:#21372a}.button[disabled]{cursor:not-allowed;opacity:.45}.result{border:1px solid #273b2f;background:#17251d;color:#e7eee3}.result-head{padding:18px 20px;border-bottom:1px solid #425347;display:flex;justify-content:space-between;gap:16px}.result h2{margin:0;font:500 1.45rem/1.1 Georgia,serif}.result p{margin:.35rem 0 0;color:#b9c8bb}.result pre{margin:0;padding:18px 20px;max-height:620px;overflow:auto;font:12px/1.55 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;overflow-wrap:anywhere}.error{border-color:#8d4d44;background:#4a211c}.error .result-head{border-color:#75433c}.htmx-indicator{display:none}.htmx-request .htmx-indicator,.htmx-request.htmx-indicator{display:inline}.footer{display:flex;justify-content:space-between;gap:24px;margin-top:34px;padding-top:20px;border-top:1px solid #9aaa9d;color:#617067;font-size:.78rem;line-height:1.5}@media(max-width:900px){.status-grid{grid-template-columns:1fr 1fr}.workspace,.guardrail{grid-template-columns:1fr}.guardrail>div+div{border-left:0;border-top:1px solid #526056}}@media(max-width:580px){.shell{width:min(100% - 20px,1240px)}.masthead{align-items:flex-start;flex-direction:column}.status-grid,.form-grid{grid-template-columns:1fr}.field.full{grid-column:auto}.footer{flex-direction:column}}
"#;

pub fn dashboard(
    environment: Environment,
    api_health: &Result<Value, String>,
    sources: &Result<Value, String>,
) -> Markup {
    let source_count = sources
        .as_ref()
        .ok()
        .and_then(Value::as_array)
        .map_or("unavailable".to_owned(), |items| items.len().to_string());
    let api_status = api_health
        .as_ref()
        .ok()
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unavailable");
    let semantic_mode = api_health
        .as_ref()
        .ok()
        .and_then(|value| value.get("query_embedding_mode"))
        .and_then(Value::as_str)
        .unwrap_or("unavailable");

    layout(
        "Embedded Alerts operator console",
        environment,
        html! {
            section class="status-grid" aria-label="runtime status" {
                (metric("API", api_status, "durable boundary"))
                (metric("Sources", &source_count, "tenant-scoped"))
                (metric("Semantic query", semantic_mode, "model-versioned"))
                (metric("Delivery", "disabled", "DEN-3460 gate"))
            }
            section class="guardrail" {
                div {
                    h2 { "Own the index; borrow discovery." }
                    p { "Search engines, feeds, and sitemaps may suggest candidate URLs. A URL is not trusted until the ingestion worker applies the registered domain policy, robots rules, canonicalization, public-network checks, bounded fetch, revision hashing, and local semantic scoring." }
                }
                div {
                    ol {
                        li { "Register an explicit public source policy." }
                        li { "Discover and fetch only through the bounded worker." }
                        li { "Create reviewable match candidates; never send from the crawler." }
                    }
                }
            }
            div class="workspace" {
                div class="stack" {
                    section class="panel" {
                        div class="panel-head" {
                            div { h2 { "Registered source policies" } p { "The PostgreSQL API is authoritative; this console does not maintain a second policy model." } }
                            span class="tag" { (source_count) }
                        }
                        div class="panel-body" { (source_list(sources)) }
                    }
                    section class="panel" {
                        div class="panel-head" {
                            div { h2 { "Create a source policy" } p { "Paste a CreateSourcePolicy document from the eal-interfaces contract. Server-owned tenant and identity fields are rejected locally." } }
                            span class="tag warn" { "starts disabled" }
                        }
                        div class="panel-body" {
                            form hx-post="/sources" hx-target="#operation-result" hx-swap="outerHTML" method="post" {
                                div class="field" {
                                    label for="policy_json" { "Contract JSON" }
                                    textarea id="policy_json" name="policy_json" required placeholder=r#"{
  "name": "Public documentation",
  "scheme": "https",
  "host": "docs.example.com",
  "respect_robots": true,
  "enabled": false
}"# {}
                                    span class="hint" { "The API performs canonical contract validation and tenant binding. Register first, then enable only after policy review." }
                                }
                                div class="actions" {
                                    button class="button" type="submit" { "Register policy" }
                                    span class="htmx-indicator" { "Submitting…" }
                                }
                            }
                        }
                    }
                    section class="panel" {
                        div class="panel-head" {
                            div { h2 { "Crawler control" } p { "Deliberately unavailable in the web process." } }
                            span class="tag warn" { "fail closed" }
                        }
                        div class="panel-body" {
                            p { "A registered source is not a license to crawl. Queue leasing, robots snapshots, DNS re-resolution, bounded fetch, canonical revision storage, and embedding must run in eal-sync. Until that durable loop is connected, no scan endpoint is exposed here." }
                            div class="actions" { button class="button secondary" type="button" disabled { "Scan now unavailable" } }
                        }
                    }
                }
                div class="stack" {
                    section class="panel" {
                        div class="panel-head" {
                            div { h2 { "Semantic search" } p { "Natural-language query decomposition and embedding stay server-side." } }
                            span class="tag" { "read only" }
                        }
                        div class="panel-body" { (semantic_search_form()) }
                    }
                    section class="panel" {
                        div class="panel-head" {
                            div { h2 { "Create match candidates" } p { "Persists candidate rows for review; it never dispatches a notification." } }
                            span class="tag warn" { "no send" }
                        }
                        div class="panel-body" { (semantic_evaluate_form()) }
                    }
                    div id="operation-result" { (result_placeholder()) }
                    section class="panel" {
                        div class="panel-head" { div { h2 { "API health detail" } p { "Useful for model-space and fail-closed readiness checks." } } }
                        (json_or_error(api_health))
                    }
                }
            }
        },
    )
}

pub fn operation_result(title: &str, value: &Value) -> Markup {
    html! {
        section id="operation-result" class="result" aria-live="polite" {
            div class="result-head" { div { h2 { (title) } p { "Returned by the canonical tenant-scoped API." } } span class="tag" { "complete" } }
            pre { (pretty(value)) }
        }
    }
}

pub fn operation_error(title: &str, message: &str) -> Markup {
    html! {
        section id="operation-result" class="result error" role="alert" aria-live="assertive" {
            div class="result-head" { div { h2 { (title) } p { "The operation did not cross the safety boundary." } } span class="tag warn" { "rejected" } }
            pre { (message) }
        }
    }
}

pub fn health(value: &Value) -> Markup {
    html! { pre class="json" { (pretty(value)) } }
}

fn layout(title: &str, environment: Environment, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="referrer" content="no-referrer";
                title { (title) }
                style { (PreEscaped(CSS)) }
                script src="https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js" defer {}
            }
            body {
                main class="shell" {
                    header class="masthead" {
                        div { div class="eyebrow" { "Domain-scoped semantic monitoring" } h1 class="brand" { "Embedded Alerts" } }
                        span class="env" { span class="dot" {} (environment.label()) }
                    }
                    (content)
                    footer class="footer" {
                        span { "Mash/Maud + Axum + HTMX operator surface" }
                        span { "Rust business logic remains in eal-api, eal-sync, eal-interfaces, and eal-libs." }
                    }
                }
            }
        }
    }
}

fn metric(label: &str, value: &str, note: &str) -> Markup {
    html! { div class="metric" { small { (label) } b { (value) } small { (note) } } }
}

fn source_list(sources: &Result<Value, String>) -> Markup {
    match sources {
        Err(message) => html! { div class="empty" { "Unable to load source policies: " (message) } },
        Ok(value) => match value.as_array() {
            Some(items) if items.is_empty() => html! { div class="empty" { "No source policies are registered for this tenant." } },
            Some(items) => html! {
                div class="source-list" {
                    @for source in items {
                        details class="source" {
                            summary {
                                div {
                                    strong { (source_title(source)) }
                                    code { (source_id(source)) }
                                }
                                span class="tag" { (source_state(source)) }
                            }
                            pre { (pretty(source)) }
                        }
                    }
                }
            },
            None => html! { div class="empty" { "The API source response was not an array." } },
        },
    }
}

fn semantic_search_form() -> Markup {
    html! {
        form hx-post="/semantic/search" hx-target="#operation-result" hx-swap="outerHTML" method="post" {
            div class="form-grid" {
                div class="field full" { label for="search_query" { "Interest" } textarea id="search_query" name="query_text" required placeholder="Notify me when a public source announces a production-ready Rust package registry feature." {} }
                div class="field" { label for="min_similarity" { "Minimum similarity" } input id="min_similarity" name="min_similarity" type="number" min="0" max="1" step="0.01" value="0.78" required; }
                div class="field" { label for="search_limit" { "Result limit" } input id="search_limit" name="limit" type="number" min="1" max="200" value="50" required; }
                div class="field full" { label for="search_sources" { "Optional source UUIDs" } input id="search_sources" name="source_ids" placeholder="comma or whitespace separated"; span class="hint" { "Leave empty to search all indexed sources visible to this tenant." } }
            }
            div class="actions" { button class="button" type="submit" { "Run semantic search" } span class="htmx-indicator" { "Searching…" } }
        }
    }
}

fn semantic_evaluate_form() -> Markup {
    html! {
        form hx-post="/semantic/evaluate" hx-target="#operation-result" hx-swap="outerHTML" method="post" {
            div class="form-grid" {
                div class="field full" { label for="alert_rule_id" { "Alert-rule UUID" } input id="alert_rule_id" name="alert_rule_id" required placeholder="00000000-0000-0000-0000-000000000000"; }
                div class="field full" { label for="candidate_query" { "Immutable rule query" } textarea id="candidate_query" name="query_text" required placeholder="Public launches of model-versioned semantic indexing for Rust applications." {} }
                div class="field" { label for="candidate_threshold" { "Candidate threshold" } input id="candidate_threshold" name="threshold" type="number" min="0" max="1" step="0.01" value="0.78" required; }
                div class="field" { label for="candidate_similarity" { "Search floor" } input id="candidate_similarity" name="min_similarity" type="number" min="0" max="1" step="0.01" value="0.70" required; }
                div class="field" { label for="candidate_limit" { "Candidate limit" } input id="candidate_limit" name="limit" type="number" min="1" max="200" value="50" required; }
                div class="field" { label for="candidate_sources" { "Optional source UUIDs" } input id="candidate_sources" name="source_ids" placeholder="comma or whitespace separated"; }
            }
            div class="actions" { button class="button" type="submit" { "Create candidates" } span class="htmx-indicator" { "Evaluating…" } }
        }
    }
}

fn result_placeholder() -> Markup {
    html! { section id="operation-result" class="empty" aria-live="polite" { "Semantic results and candidate receipts appear here." } }
}

fn json_or_error(value: &Result<Value, String>) -> Markup {
    match value {
        Ok(value) => html! { pre class="json" { (pretty(value)) } },
        Err(message) => html! { pre class="json" { (message) } },
    }
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_owned())
}

fn source_title(source: &Value) -> String {
    for key in ["name", "host", "domain", "hostname", "origin"] {
        if let Some(value) = source.get(key).and_then(Value::as_str) {
            return value.to_owned();
        }
    }
    "Source policy".to_owned()
}

fn source_id(source: &Value) -> String {
    source
        .get("id")
        .or_else(|| source.get("source_id"))
        .and_then(Value::as_str)
        .unwrap_or("identity unavailable")
        .to_owned()
}

fn source_state(source: &Value) -> &'static str {
    if source.get("enabled").and_then(Value::as_bool) == Some(true) {
        "enabled"
    } else {
        "disabled"
    }
}
