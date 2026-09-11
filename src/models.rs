use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DiscoveryMode {
    Seed,
    RobotsSitemap,
    Sitemap,
    Rss,
    LinkCrawl,
    ExternalIndex,
}

impl DiscoveryMode {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Seed => "Seed URLs",
            Self::RobotsSitemap => "robots.txt sitemaps",
            Self::Sitemap => "Conventional sitemap",
            Self::Rss => "RSS / Atom",
            Self::LinkCrawl => "Same-domain links",
            Self::ExternalIndex => "External candidates",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SourceDomain {
    pub(crate) id: Uuid,
    pub(crate) tenant_id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) name: String,
    pub(crate) base_url: String,
    pub(crate) host: String,
    pub(crate) include_subdomains: bool,
    pub(crate) seed_urls: Vec<String>,
    pub(crate) discovery_modes: Vec<DiscoveryMode>,
    pub(crate) max_pages_per_scan: usize,
    pub(crate) source_priority: f32,
    pub(crate) respect_robots: bool,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CreateSourceDomain {
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) include_subdomains: bool,
    pub(crate) seed_urls: Vec<String>,
    pub(crate) discovery_modes: Vec<DiscoveryMode>,
    pub(crate) max_pages_per_scan: usize,
    pub(crate) source_priority: f32,
    pub(crate) respect_robots: bool,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SourceForm {
    pub(crate) name: String,
    pub(crate) domain: String,
    #[serde(default)]
    pub(crate) seed_urls: String,
    #[serde(default)]
    pub(crate) include_subdomains: Option<String>,
    #[serde(default)]
    pub(crate) mode_seed: Option<String>,
    #[serde(default)]
    pub(crate) mode_robots_sitemap: Option<String>,
    #[serde(default)]
    pub(crate) mode_sitemap: Option<String>,
    #[serde(default)]
    pub(crate) mode_rss: Option<String>,
    #[serde(default)]
    pub(crate) mode_link_crawl: Option<String>,
    #[serde(default)]
    pub(crate) mode_external_index: Option<String>,
    pub(crate) max_pages_per_scan: usize,
    pub(crate) source_priority: f32,
    #[serde(default)]
    pub(crate) respect_robots: Option<String>,
    #[serde(default)]
    pub(crate) enabled: Option<String>,
}

impl TryFrom<SourceForm> for CreateSourceDomain {
    type Error = String;

    fn try_from(form: SourceForm) -> Result<Self, Self::Error> {
        let name = form.name.trim().to_owned();
        if name.is_empty() || name.chars().count() > 120 {
            return Err("Source name must contain 1 to 120 characters.".into());
        }
        let domain = form.domain.trim().to_owned();
        if domain.is_empty() || domain.chars().count() > 2_048 {
            return Err("Domain must contain 1 to 2,048 characters.".into());
        }
        if !(1..=100).contains(&form.max_pages_per_scan) {
            return Err("Page budget must be between 1 and 100 pages per scan.".into());
        }
        if !form.source_priority.is_finite() || !(0.0..=1.0).contains(&form.source_priority) {
            return Err("Source priority must be a finite number between 0 and 1.".into());
        }
        if form.respect_robots.is_none() {
            return Err("robots.txt compliance is mandatory for public-page indexing.".into());
        }

        let mut discovery_modes = Vec::new();
        for (selected, mode) in [
            (form.mode_seed.is_some(), DiscoveryMode::Seed),
            (
                form.mode_robots_sitemap.is_some(),
                DiscoveryMode::RobotsSitemap,
            ),
            (form.mode_sitemap.is_some(), DiscoveryMode::Sitemap),
            (form.mode_rss.is_some(), DiscoveryMode::Rss),
            (form.mode_link_crawl.is_some(), DiscoveryMode::LinkCrawl),
            (
                form.mode_external_index.is_some(),
                DiscoveryMode::ExternalIndex,
            ),
        ] {
            if selected {
                discovery_modes.push(mode);
            }
        }
        if discovery_modes.is_empty() {
            return Err("Select at least one discovery mode.".into());
        }

        let mut seed_urls = BTreeSet::new();
        for value in form.seed_urls.lines().flat_map(|line| line.split(',')) {
            let value = value.trim();
            if !value.is_empty() {
                seed_urls.insert(value.to_owned());
            }
        }
        if seed_urls.len() > 50 {
            return Err("A source may contain at most 50 seed URLs.".into());
        }

        Ok(Self {
            name,
            domain,
            include_subdomains: form.include_subdomains.is_some(),
            seed_urls: seed_urls.into_iter().collect(),
            discovery_modes,
            max_pages_per_scan: form.max_pages_per_scan,
            source_priority: form.source_priority,
            respect_robots: true,
            enabled: form.enabled.is_some(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiHealth {
    pub(crate) service: String,
    pub(crate) status: String,
    pub(crate) environment: String,
    pub(crate) storage_mode: String,
    pub(crate) semantic_index_mode: String,
    pub(crate) production_ready: bool,
    pub(crate) database_connected: bool,
    pub(crate) supabase_configured: bool,
    pub(crate) embedding_mode: String,
    pub(crate) embedding_model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ScanReport {
    pub(crate) source_id: Uuid,
    pub(crate) discovered_urls: usize,
    pub(crate) sitemap_count: usize,
    pub(crate) attempted: usize,
    pub(crate) created: usize,
    pub(crate) updated: usize,
    pub(crate) unchanged: usize,
    pub(crate) rejected_by_robots: usize,
    pub(crate) failed: usize,
    pub(crate) failures: Vec<ScanFailure>,
    pub(crate) embedding_model: serde_json::Value,
    pub(crate) extractor_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ScanFailure {
    pub(crate) url: String,
    pub(crate) code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PageIndexRecord {
    pub(crate) id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) previous_revision_id: Option<Uuid>,
    pub(crate) canonical_url: String,
    pub(crate) fetched_at: DateTime<Utc>,
    pub(crate) content_hash: String,
    pub(crate) title: Option<String>,
    pub(crate) summary: String,
    pub(crate) keywords: Vec<String>,
    pub(crate) entities: Vec<String>,
    pub(crate) model: serde_json::Value,
    pub(crate) extractor_version: String,
    pub(crate) segment_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MatchCandidate {
    pub(crate) id: Uuid,
    pub(crate) match_key: String,
    pub(crate) tenant_id: Uuid,
    pub(crate) alert_rule_id: Uuid,
    pub(crate) alert_rule_revision: u32,
    pub(crate) page_revision_id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) canonical_url: String,
    pub(crate) content_hash: String,
    pub(crate) query_hash: String,
    pub(crate) model: serde_json::Value,
    pub(crate) score: f32,
    pub(crate) components: ScoreComponents,
    pub(crate) evidence: Vec<MatchEvidence>,
    pub(crate) state: String,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScoreComponents {
    pub(crate) semantic: f32,
    pub(crate) lexical: f32,
    pub(crate) entity: f32,
    pub(crate) recency: f32,
    pub(crate) source_priority: f32,
    pub(crate) weights: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MatchEvidence {
    pub(crate) page_segment_kind: String,
    pub(crate) page_text: String,
    pub(crate) query_segment_kind: String,
    pub(crate) similarity: f32,
    pub(crate) weighted_similarity: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchForm {
    pub(crate) query_text: String,
    pub(crate) threshold: f32,
    pub(crate) limit: usize,
    #[serde(default)]
    pub(crate) source_id: String,
    #[serde(default)]
    pub(crate) alert_rule_id: String,
    #[serde(default)]
    pub(crate) alert_rule_revision: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SemanticSearchRequest {
    pub(crate) query_text: String,
    pub(crate) source_ids: Vec<Uuid>,
    pub(crate) threshold: f32,
    pub(crate) limit: usize,
    pub(crate) cursor: Option<String>,
    pub(crate) expected_model: Option<serde_json::Value>,
    pub(crate) alert_rule: Option<AlertRuleRevisionRef>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AlertRuleRevisionRef {
    pub(crate) id: Uuid,
    pub(crate) revision: u32,
}

impl TryFrom<SearchForm> for SemanticSearchRequest {
    type Error = String;

    fn try_from(form: SearchForm) -> Result<Self, Self::Error> {
        let query_text = form.query_text.trim().to_owned();
        if !(3..=2_000).contains(&query_text.chars().count()) {
            return Err("Query must contain 3 to 2,000 characters.".into());
        }
        if !form.threshold.is_finite() || !(0.0..=1.0).contains(&form.threshold) {
            return Err("Threshold must be a finite number between 0 and 1.".into());
        }
        if !(1..=100).contains(&form.limit) {
            return Err("Result limit must be between 1 and 100.".into());
        }

        let source_ids = if form.source_id.trim().is_empty() {
            Vec::new()
        } else {
            vec![
                Uuid::parse_str(form.source_id.trim())
                    .map_err(|_| "Selected source ID is invalid.".to_owned())?,
            ]
        };

        let rule_id = form.alert_rule_id.trim();
        let rule_revision = form.alert_rule_revision.trim();
        let alert_rule = match (rule_id.is_empty(), rule_revision.is_empty()) {
            (true, true) => None,
            (false, false) => {
                let id = Uuid::parse_str(rule_id)
                    .map_err(|_| "Alert-rule ID must be a UUID.".to_owned())?;
                let revision = rule_revision
                    .parse::<u32>()
                    .map_err(|_| "Alert-rule revision must be a positive integer.".to_owned())?;
                if revision == 0 {
                    return Err("Alert-rule revision must be greater than zero.".into());
                }
                Some(AlertRuleRevisionRef { id, revision })
            }
            _ => {
                return Err(
                    "Provide both alert-rule ID and revision to create candidates, or leave both blank."
                        .into(),
                );
            }
        };

        Ok(Self {
            query_text,
            source_ids,
            threshold: form.threshold,
            limit: form.limit,
            cursor: None,
            expected_model: None,
            alert_rule,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SemanticSearchResponse {
    pub(crate) query_text: String,
    pub(crate) model: serde_json::Value,
    pub(crate) results: Vec<SearchResult>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) compared_pages: usize,
    pub(crate) skipped_cross_model_pages: usize,
    pub(crate) candidate_matches_created: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResult {
    pub(crate) page_revision_id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) summary: String,
    pub(crate) fetched_at: DateTime<Utc>,
    pub(crate) content_hash: String,
    pub(crate) model: serde_json::Value,
    pub(crate) score: f32,
    pub(crate) components: ScoreComponents,
    pub(crate) evidence: Vec<MatchEvidence>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_form() -> SourceForm {
        SourceForm {
            name: "Rust releases".into(),
            domain: "blog.rust-lang.org".into(),
            seed_urls: "https://blog.rust-lang.org/\nhttps://blog.rust-lang.org/".into(),
            include_subdomains: None,
            mode_seed: Some("on".into()),
            mode_robots_sitemap: Some("on".into()),
            mode_sitemap: None,
            mode_rss: None,
            mode_link_crawl: None,
            mode_external_index: None,
            max_pages_per_scan: 25,
            source_priority: 0.6,
            respect_robots: Some("on".into()),
            enabled: Some("on".into()),
        }
    }

    #[test]
    fn source_form_deduplicates_seeds_and_forces_robots() {
        let source = CreateSourceDomain::try_from(source_form()).expect("valid source");
        assert_eq!(source.seed_urls.len(), 1);
        assert!(source.respect_robots);
        assert_eq!(
            source.discovery_modes,
            vec![DiscoveryMode::Seed, DiscoveryMode::RobotsSitemap]
        );
    }

    #[test]
    fn source_form_rejects_robots_opt_out() {
        let mut form = source_form();
        form.respect_robots = None;
        let error = CreateSourceDomain::try_from(form).expect_err("robots must be required");
        assert!(error.contains("mandatory"));
    }

    #[test]
    fn candidate_creation_requires_an_immutable_rule_revision_pair() {
        let form = SearchForm {
            query_text: "Acme launches renewable energy tools".into(),
            threshold: 0.72,
            limit: 20,
            source_id: String::new(),
            alert_rule_id: Uuid::new_v4().to_string(),
            alert_rule_revision: String::new(),
        };
        let error = SemanticSearchRequest::try_from(form).expect_err("pair must be complete");
        assert!(error.contains("both"));
    }
}
