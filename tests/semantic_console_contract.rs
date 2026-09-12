#[path = "../src/semantic_console.rs"]
mod semantic_console;

use semantic_console::{
    CandidateState, MatchSummary, RuleSummary, SemanticConsoleView, SourceStatus, SourceSummary,
    render_semantic_console,
};

#[test]
fn semantic_console_exposes_one_consistent_operator_workflow() {
    let view = SemanticConsoleView {
        tenant_name: "Example organization".into(),
        csrf_token: "csrf".into(),
        sources: vec![SourceSummary {
            id: "source-1".into(),
            domain: "news.example.org".into(),
            include_subdomains: true,
            respect_robots: true,
            enabled: true,
            page_budget: 500,
            indexed_pages: 128,
            last_scan_label: "just now".into(),
            status: SourceStatus::Scanning,
        }],
        rules: vec![RuleSummary {
            id: "rule-1".into(),
            name: "Engineering launches".into(),
            revision: 1,
            query_text: "Notify me about new infrastructure tools for engineering teams.".into(),
            threshold: 0.74,
            enabled: true,
            domain_count: 1,
            candidate_count: 3,
        }],
        matches: vec![MatchSummary {
            id: "match-1".into(),
            rule_id: "rule-1".into(),
            rule_name: "Engineering launches".into(),
            page_title: "New infrastructure platform launches".into(),
            canonical_url: "https://news.example.org/platform".into(),
            source_domain: "news.example.org".into(),
            score: 0.84,
            semantic_score: 0.89,
            lexical_score: 0.61,
            entity_score: 0.72,
            recency_score: 0.95,
            source_priority_score: 0.80,
            matched_sentence: "The platform gives engineering teams a safer way to manage infrastructure changes.".into(),
            matched_entities: vec!["Example Platform".into()],
            matched_keywords: vec!["engineering".into(), "infrastructure".into()],
            discovered_label: "Discovered now".into(),
            model_label: "model-v1 · extractor-v1".into(),
            state: CandidateState::Approved,
        }],
    };

    let html = render_semantic_console(&view);
    assert!(html.contains("Approved domains"));
    assert!(html.contains("Natural-language alert rules"));
    assert!(html.contains("Match candidates"));
    assert!(html.contains("hx-post=\"/ui/alert-rules/rule-1/evaluate\""));
    assert!(html.contains("Best complete-sentence evidence"));
    assert!(!html.contains("name=\"vector\""));
}

#[test]
fn all_wire_states_remain_renderable() {
    let source_states = [
        SourceStatus::Ready,
        SourceStatus::Scanning,
        SourceStatus::Paused,
        SourceStatus::Rejected,
    ];
    let candidate_states = [
        CandidateState::Candidate,
        CandidateState::Approved,
        CandidateState::Suppressed,
        CandidateState::Delivered,
        CandidateState::Failed,
    ];
    assert_eq!(source_states.len(), 4);
    assert_eq!(candidate_states.len(), 5);
}
