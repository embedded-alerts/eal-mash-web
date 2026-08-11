use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticConsoleView {
    pub tenant_name: String,
    pub sources: Vec<SourceSummary>,
    pub rules: Vec<RuleSummary>,
    pub matches: Vec<MatchSummary>,
    pub csrf_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSummary {
    pub id: String,
    pub domain: String,
    pub include_subdomains: bool,
    pub respect_robots: bool,
    pub enabled: bool,
    pub page_budget: u32,
    pub indexed_pages: u64,
    pub last_scan_label: String,
    pub status: SourceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceStatus {
    Ready,
    Scanning,
    Paused,
    Rejected,
}

impl SourceStatus {
    const fn wire_name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Scanning => "scanning",
            Self::Paused => "paused",
            Self::Rejected => "rejected",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Scanning => "Scanning",
            Self::Paused => "Paused",
            Self::Rejected => "Needs review",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSummary {
    pub id: String,
    pub name: String,
    pub revision: u32,
    pub query_text: String,
    pub threshold: f32,
    pub enabled: bool,
    pub domain_count: u32,
    pub candidate_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchSummary {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub page_title: String,
    pub canonical_url: String,
    pub source_domain: String,
    pub score: f32,
    pub semantic_score: f32,
    pub lexical_score: f32,
    pub entity_score: f32,
    pub recency_score: f32,
    pub source_priority_score: f32,
    pub matched_sentence: String,
    pub matched_entities: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub discovered_label: String,
    pub model_label: String,
    pub state: CandidateState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateState {
    Candidate,
    Approved,
    Suppressed,
    Delivered,
    Failed,
}

impl CandidateState {
    const fn wire_name(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Approved => "approved",
            Self::Suppressed => "suppressed",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Candidate => "Candidate",
            Self::Approved => "Approved",
            Self::Suppressed => "Suppressed",
            Self::Delivered => "Delivered",
            Self::Failed => "Delivery failed",
        }
    }
}

pub fn render_semantic_console(view: &SemanticConsoleView) -> String {
    let mut html = String::with_capacity(32_000);
    let tenant_name = escape_html(&view.tenant_name);
    write!(
        html,
        r#"<main class="semantic-console" data-component="semantic-console">
<header class="console-header">
  <div>
    <p class="eyebrow">Embedded Alerts / {tenant_name}</p>
    <h1>Semantic monitoring console</h1>
    <p class="lede">Index approved public domains, describe what matters in natural language, and review explainable matches before delivery.</p>
  </div>
  <nav aria-label="Semantic console sections" class="section-nav">
    <a href="#sources">Sources</a>
    <a href="#rules">Alert rules</a>
    <a href="#matches">Match candidates</a>
  </nav>
</header>
<div id="console-flash" class="flash-region" role="status" aria-live="polite"></div>
"#
    )
    .expect("writing to String cannot fail");

    render_source_section(&mut html, view);
    render_rule_section(&mut html, view);
    render_match_section(&mut html, view);
    html.push_str("</main>");
    html
}

fn render_source_section(html: &mut String, view: &SemanticConsoleView) {
    let csrf_token = escape_attr(&view.csrf_token);
    write!(
        html,
        r#"<section id="sources" class="console-section" aria-labelledby="sources-title">
<div class="section-heading">
  <div><p class="section-kicker">Discovery boundary</p><h2 id="sources-title">Approved domains</h2></div>
  <p>Only configured public hosts are eligible. External indexes may suggest URLs, but every page is fetched and checked against this policy.</p>
</div>
<form class="source-form panel" method="post" action="/ui/sources" hx-post="/ui/sources" hx-target="#source-list" hx-swap="outerHTML" hx-disabled-elt="find button" hx-indicator="#source-submit-progress">
  <input type="hidden" name="csrf_token" value="{csrf_token}">
  <div class="field-grid">
    <label><span>Public domain</span><input name="domain" type="text" inputmode="url" autocomplete="off" placeholder="news.example.org" pattern="^[A-Za-z0-9.-]+$" maxlength="253" required aria-describedby="domain-help"></label>
    <label><span>Seed URLs</span><textarea name="seed_urls" rows="3" maxlength="8000" placeholder="https://news.example.org/\nhttps://news.example.org/sitemap.xml" aria-describedby="seed-help"></textarea></label>
    <label><span>Pages per scan</span><input name="page_budget" type="number" min="1" max="5000" value="250" required></label>
    <label><span>Source priority</span><input name="priority" type="number" min="0" max="100" value="50" required></label>
  </div>
  <p id="domain-help" class="field-help">IP literals, localhost, credentials in URLs, private networks, and off-domain redirects are rejected.</p>
  <p id="seed-help" class="field-help">One HTTPS URL per line. Seeds must remain inside the registered host policy.</p>
  <fieldset class="choice-row"><legend>Policy</legend>
    <label><input type="checkbox" name="include_subdomains" value="true"> Include subdomains</label>
    <label><input type="checkbox" name="respect_robots" value="true" checked> Enforce robots.txt</label>
    <label><input type="checkbox" name="discover_sitemaps" value="true" checked> Discover sitemaps</label>
    <label><input type="checkbox" name="discover_links" value="true" checked> Follow bounded same-domain links</label>
  </fieldset>
  <div class="form-actions"><button class="primary-action" type="submit">Register source</button><span id="source-submit-progress" class="htmx-indicator" aria-live="polite">Registering…</span></div>
</form>
"#
    )
    .expect("writing to String cannot fail");

    render_source_list(html, &view.sources, &view.csrf_token);
    html.push_str("</section>");
}

pub fn render_source_list(html: &mut String, sources: &[SourceSummary], csrf_token: &str) {
    html.push_str(r#"<div id="source-list" class="card-grid source-list">"#);
    if sources.is_empty() {
        html.push_str(
            r#"<article class="empty-state"><h3>No domains registered</h3><p>Add the first public domain to begin bounded discovery.</p></article>"#,
        );
    }
    for source in sources {
        let id = escape_attr(&source.id);
        let domain = escape_html(&source.domain);
        let status = source.status.wire_name();
        let status_label = source.status.label();
        let subdomains = if source.include_subdomains {
            "Exact host + subdomains"
        } else {
            "Exact host only"
        };
        let robots = if source.respect_robots {
            "Robots enforced"
        } else {
            "Robots disabled"
        };
        let enabled_label = if source.enabled { "Disable" } else { "Enable" };
        let enabled_action = if source.enabled { "disable" } else { "enable" };
        let last_scan = escape_html(&source.last_scan_label);
        let csrf = escape_attr(csrf_token);
        write!(
            html,
            r#"<article class="source-card panel" data-source-id="{id}" data-status="{status}">
  <div class="card-heading"><div><p class="domain-label">{domain}</p><h3>{subdomains}</h3></div><span class="status-pill status-{status}">{status_label}</span></div>
  <dl class="metric-list"><div><dt>Indexed pages</dt><dd>{indexed_pages}</dd></div><div><dt>Scan budget</dt><dd>{page_budget}</dd></div><div><dt>Robots</dt><dd>{robots}</dd></div><div><dt>Last scan</dt><dd>{last_scan}</dd></div></dl>
  <div class="card-actions">
    <button type="button" hx-post="/ui/sources/{id}/scan" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="outerHTML" hx-disabled-elt="this">Scan now</button>
    <button type="button" hx-post="/ui/sources/{id}/{enabled_action}" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="outerHTML" hx-confirm="{enabled_label} indexing for {domain}?">{enabled_label}</button>
    <a href="/sources/{id}">Inspect pages</a>
  </div>
</article>"#,
            indexed_pages = source.indexed_pages,
            page_budget = source.page_budget,
        )
        .expect("writing to String cannot fail");
    }
    html.push_str("</div>");
}

fn render_rule_section(html: &mut String, view: &SemanticConsoleView) {
    let csrf_token = escape_attr(&view.csrf_token);
    write!(
        html,
        r#"<section id="rules" class="console-section" aria-labelledby="rules-title">
<div class="section-heading">
  <div><p class="section-kicker">Semantic intent</p><h2 id="rules-title">Natural-language alert rules</h2></div>
  <p>The complete sentence remains the strongest query representation. Keywords and proper nouns are companion evidence, not replacements.</p>
</div>
<form class="rule-form panel" method="post" action="/ui/alert-rules" hx-post="/ui/alert-rules" hx-target="#rule-list" hx-swap="outerHTML" hx-disabled-elt="find button" hx-indicator="#rule-submit-progress">
  <input type="hidden" name="csrf_token" value="{csrf_token}">
  <div class="field-grid">
    <label><span>Rule name</span><input name="name" type="text" maxlength="120" placeholder="Colombia energy launches" required></label>
    <label class="wide-field"><span>What should Embedded Alerts find?</span><textarea name="query_text" rows="4" minlength="3" maxlength="700" placeholder="Notify me when a Colombian renewable-energy company launches tooling for engineering teams." required aria-describedby="query-help"></textarea></label>
    <label><span>Candidate threshold</span><input name="threshold" type="number" min="0" max="1" step="0.01" value="0.72" required></label>
    <label><span>Domain scope</span><select name="source_scope" required><option value="all">All approved domains</option><option value="selected">Selected domains only</option></select></label>
  </div>
  <p id="query-help" class="field-help">Use a complete thought. The server derives keyword and entity views and owns all vector generation.</p>
  <div class="form-actions"><button class="primary-action" type="submit">Create immutable revision</button><button type="button" hx-post="/ui/query-preview" hx-include="closest form" hx-target="#query-preview" hx-swap="innerHTML">Preview semantic views</button><span id="rule-submit-progress" class="htmx-indicator" aria-live="polite">Saving…</span></div>
  <div id="query-preview" class="query-preview" aria-live="polite"></div>
</form>
"#
    )
    .expect("writing to String cannot fail");

    html.push_str(r#"<div id="rule-list" class="card-grid rule-list">"#);
    if view.rules.is_empty() {
        html.push_str(
            r#"<article class="empty-state"><h3>No alert rules</h3><p>Describe the first interest using a complete natural-language statement.</p></article>"#,
        );
    }
    for rule in &view.rules {
        render_rule_card(html, rule, &view.csrf_token);
    }
    html.push_str("</div></section>");
}

fn render_rule_card(html: &mut String, rule: &RuleSummary, csrf_token: &str) {
    let id = escape_attr(&rule.id);
    let name = escape_html(&rule.name);
    let query_text = escape_html(&rule.query_text);
    let state = if rule.enabled { "enabled" } else { "paused" };
    let action = if rule.enabled { "pause" } else { "enable" };
    let action_label = if rule.enabled { "Pause" } else { "Enable" };
    let csrf = escape_attr(csrf_token);
    let threshold = format_score(rule.threshold);
    write!(
        html,
        r#"<article class="rule-card panel" data-rule-id="{id}" data-state="{state}">
  <div class="card-heading"><div><p class="revision-label">Revision {revision}</p><h3>{name}</h3></div><span class="status-pill status-{state}">{state}</span></div>
  <blockquote>{query_text}</blockquote>
  <dl class="metric-list"><div><dt>Threshold</dt><dd>{threshold}</dd></div><div><dt>Domains</dt><dd>{domain_count}</dd></div><div><dt>Candidates</dt><dd>{candidate_count}</dd></div></dl>
  <div class="card-actions">
    <button type="button" hx-post="/ui/alert-rules/{id}/evaluate" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="#match-list" hx-swap="afterbegin" hx-disabled-elt="this">Evaluate new pages</button>
    <button type="button" hx-post="/ui/alert-rules/{id}/{action}" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="outerHTML">{action_label}</button>
    <a href="/alert-rules/{id}">Revision history</a>
  </div>
</article>"#,
        revision = rule.revision,
        domain_count = rule.domain_count,
        candidate_count = rule.candidate_count,
    )
    .expect("writing to String cannot fail");
}

fn render_match_section(html: &mut String, view: &SemanticConsoleView) {
    write!(
        html,
        r#"<section id="matches" class="console-section" aria-labelledby="matches-title">
<div class="section-heading">
  <div><p class="section-kicker">Explainable ranking</p><h2 id="matches-title">Match candidates</h2></div>
  <p>Review semantic, lexical, entity, recency, and source-priority evidence before a candidate enters the delivery state machine.</p>
</div>
<div class="filter-bar panel" role="search">
  <label><span>State</span><select name="state" hx-get="/ui/matches" hx-target="#match-list" hx-include="closest div" hx-trigger="change"><option value="candidate">Candidate</option><option value="approved">Approved</option><option value="suppressed">Suppressed</option><option value="delivered">Delivered</option><option value="all">All</option></select></label>
  <label><span>Minimum score</span><input name="minimum_score" type="number" min="0" max="1" step="0.01" value="0.70" hx-get="/ui/matches" hx-target="#match-list" hx-include="closest div" hx-trigger="change delay:250ms"></label>
  <label><span>Search titles and evidence</span><input name="query" type="search" maxlength="200" placeholder="energy, Colombia, launch…" hx-get="/ui/matches" hx-target="#match-list" hx-include="closest div" hx-trigger="input changed delay:350ms, search"></label>
</div>
<div id="match-list" class="match-list" aria-live="polite">
"#
    )
    .expect("writing to String cannot fail");
    if view.matches.is_empty() {
        html.push_str(
            r#"<article class="empty-state"><h3>No candidates yet</h3><p>Scan an approved source, then evaluate an enabled rule.</p></article>"#,
        );
    }
    for candidate in &view.matches {
        render_match_card(html, candidate, &view.csrf_token);
    }
    html.push_str("</div></section>");
}

pub fn render_match_card(html: &mut String, candidate: &MatchSummary, csrf_token: &str) {
    let id = escape_attr(&candidate.id);
    let rule_id = escape_attr(&candidate.rule_id);
    let rule_name = escape_html(&candidate.rule_name);
    let page_title = escape_html(&candidate.page_title);
    let canonical_url = escape_attr(&candidate.canonical_url);
    let source_domain = escape_html(&candidate.source_domain);
    let sentence = escape_html(&candidate.matched_sentence);
    let discovered = escape_html(&candidate.discovered_label);
    let model = escape_html(&candidate.model_label);
    let state = candidate.state.wire_name();
    let state_label = candidate.state.label();
    let csrf = escape_attr(csrf_token);
    let score = clamp_score(candidate.score);
    let score_percent = (score * 100.0).round() as u32;

    write!(
        html,
        r#"<article class="match-card panel" data-match-id="{id}" data-rule-id="{rule_id}" data-state="{state}">
  <div class="match-heading">
    <div><p class="match-context">{rule_name} · {source_domain}</p><h3><a href="{canonical_url}" rel="noopener noreferrer" target="_blank">{page_title}</a></h3></div>
    <div class="score-badge" aria-label="Overall match score {score_percent} percent"><strong>{score_percent}</strong><span>/ 100</span></div>
  </div>
  <div class="score-meter" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{score_percent}" aria-label="Overall semantic match"><span style="--score:{score_percent}%"></span></div>
  <blockquote class="sentence-evidence"><span>Best complete-sentence evidence</span>{sentence}</blockquote>
  <div class="evidence-grid">
    <section aria-label="Score components"><h4>Why it matched</h4><dl class="score-components">"#
    )
    .expect("writing to String cannot fail");

    render_score_component(html, "Semantic", candidate.semantic_score);
    render_score_component(html, "Lexical", candidate.lexical_score);
    render_score_component(html, "Entity", candidate.entity_score);
    render_score_component(html, "Recency", candidate.recency_score);
    render_score_component(html, "Source priority", candidate.source_priority_score);
    html.push_str("</dl></section><section aria-label=\"Matched concepts\"><h4>Concept evidence</h4>");
    render_tag_group(html, "Entities", &candidate.matched_entities, "entity-tag");
    render_tag_group(html, "Keywords", &candidate.matched_keywords, "keyword-tag");
    html.push_str("</section></div>");

    write!(
        html,
        r#"<footer class="match-footer">
  <div><span class="status-pill status-{state}">{state_label}</span><span>{discovered}</span><span>{model}</span></div>
  <div class="card-actions">
    <button type="button" class="primary-action" hx-post="/ui/matches/{id}/approve" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="outerHTML" hx-confirm="Approve this candidate for the delivery state machine?">Approve</button>
    <button type="button" hx-post="/ui/matches/{id}/suppress" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="outerHTML">Suppress</button>
    <button type="button" hx-post="/ui/matches/{id}/dismiss" hx-vals='{{"csrf_token":"{csrf}"}}' hx-target="closest article" hx-swap="delete">Dismiss</button>
    <a href="/matches/{id}">Full evidence</a>
  </div>
</footer></article>"#
    )
    .expect("writing to String cannot fail");
}

fn render_score_component(html: &mut String, label: &str, score: f32) {
    let score = clamp_score(score);
    let score_percent = (score * 100.0).round() as u32;
    let label = escape_html(label);
    write!(
        html,
        r#"<div><dt>{label}</dt><dd><span class="mini-meter" aria-hidden="true"><i style="--score:{score_percent}%"></i></span><span>{score_percent}%</span></dd></div>"#
    )
    .expect("writing to String cannot fail");
}

fn render_tag_group(html: &mut String, label: &str, values: &[String], class_name: &str) {
    let label = escape_html(label);
    write!(html, r#"<div class="tag-group"><span>{label}</span><ul>"#)
        .expect("writing to String cannot fail");
    if values.is_empty() {
        html.push_str("<li class=\"muted-tag\">None</li>");
    } else {
        for value in values {
            let value = escape_html(value);
            let class_name = escape_attr(class_name);
            write!(html, r#"<li class="{class_name}">{value}</li>"#)
                .expect("writing to String cannot fail");
        }
    }
    html.push_str("</ul></div>");
}

fn clamp_score(score: f32) -> f32 {
    if score.is_finite() {
        score.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn format_score(score: f32) -> String {
    format!("{:.0}%", clamp_score(score) * 100.0)
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_view() -> SemanticConsoleView {
        SemanticConsoleView {
            tenant_name: "Acme & Partners".into(),
            csrf_token: "csrf-token".into(),
            sources: vec![SourceSummary {
                id: "source-1".into(),
                domain: "news.example.org".into(),
                include_subdomains: false,
                respect_robots: true,
                enabled: true,
                page_budget: 250,
                indexed_pages: 42,
                last_scan_label: "2 minutes ago".into(),
                status: SourceStatus::Ready,
            }],
            rules: vec![RuleSummary {
                id: "rule-1".into(),
                name: "Energy launches".into(),
                revision: 3,
                query_text: "Notify me when a Colombian renewable-energy company launches tooling for engineering teams.".into(),
                threshold: 0.72,
                enabled: true,
                domain_count: 1,
                candidate_count: 7,
            }],
            matches: vec![MatchSummary {
                id: "match-1".into(),
                rule_id: "rule-1".into(),
                rule_name: "Energy launches".into(),
                page_title: "Acme launches Atlas".into(),
                canonical_url: "https://news.example.org/acme-atlas".into(),
                source_domain: "news.example.org".into(),
                score: 0.89,
                semantic_score: 0.93,
                lexical_score: 0.71,
                entity_score: 0.88,
                recency_score: 0.94,
                source_priority_score: 0.65,
                matched_sentence: "Acme Corporation launched Atlas for renewable-energy engineering teams in Colombia.".into(),
                matched_entities: vec!["Acme Corporation".into(), "Colombia".into()],
                matched_keywords: vec!["renewable".into(), "engineering".into()],
                discovered_label: "Discovered 4 minutes ago".into(),
                model_label: "semantic-v1 · extractor-v1".into(),
                state: CandidateState::Candidate,
            }],
        }
    }

    #[test]
    fn renders_domain_policy_query_and_candidate_actions() {
        let html = render_semantic_console(&sample_view());
        for marker in [
            "name=\"domain\"",
            "name=\"respect_robots\"",
            "name=\"query_text\"",
            "Preview semantic views",
            "Best complete-sentence evidence",
            "Why it matched",
            "/approve",
            "/suppress",
            "role=\"meter\"",
        ] {
            assert!(html.contains(marker), "missing marker: {marker}");
        }
    }

    #[test]
    fn browser_never_receives_raw_vector_controls() {
        let html = render_semantic_console(&sample_view()).to_ascii_lowercase();
        assert!(!html.contains("name=\"vector\""));
        assert!(!html.contains("embedding_values"));
        assert!(!html.contains("next.js"));
    }

    #[test]
    fn untrusted_values_are_escaped() {
        let mut view = sample_view();
        view.matches[0].page_title = "<script>alert('x')</script>".into();
        view.matches[0].canonical_url = "https://example.org/?q=\" onclick=\"alert(1)".into();
        let html = render_semantic_console(&view);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("onclick=\"alert(1)"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot; onclick=&quot;"));
    }

    #[test]
    fn non_finite_scores_fail_closed_to_zero() {
        assert_eq!(clamp_score(f32::NAN), 0.0);
        assert_eq!(clamp_score(f32::INFINITY), 0.0);
        assert_eq!(clamp_score(-1.0), 0.0);
        assert_eq!(clamp_score(2.0), 1.0);
    }
}
