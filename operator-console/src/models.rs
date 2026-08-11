use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const MAX_QUERY_CHARS: usize = 2_000;
const MAX_SOURCE_POLICY_BYTES: usize = 64 * 1024;
const MAX_SOURCE_FILTERS: usize = 100;

#[derive(Debug, Deserialize)]
pub struct SourcePolicyForm {
    pub policy_json: String,
}

impl SourcePolicyForm {
    pub fn into_policy(self) -> Result<Value> {
        if self.policy_json.len() > MAX_SOURCE_POLICY_BYTES {
            bail!("source policy JSON exceeds the 64 KiB limit");
        }
        let value: Value = serde_json::from_str(&self.policy_json)
            .context("source policy must be valid JSON")?;
        let object = value
            .as_object()
            .context("source policy must be a JSON object")?;
        for forbidden in ["tenant_id", "id", "created_at", "updated_at"] {
            if object.contains_key(forbidden) {
                bail!("source policy must not set server-owned field {forbidden}");
            }
        }
        if object.is_empty() {
            bail!("source policy must not be empty");
        }
        Ok(value)
    }
}

#[derive(Debug, Deserialize)]
pub struct SemanticSearchForm {
    pub query_text: String,
    #[serde(default = "default_similarity")]
    pub min_similarity: f32,
    #[serde(default = "default_limit")]
    pub limit: u16,
    #[serde(default)]
    pub source_ids: String,
}

impl SemanticSearchForm {
    pub fn into_payload(self) -> Result<SemanticSearchPayload> {
        validate_query(&self.query_text)?;
        validate_score("min_similarity", self.min_similarity)?;
        if !(1..=200).contains(&self.limit) {
            bail!("limit must be between 1 and 200");
        }
        Ok(SemanticSearchPayload {
            query_text: self.query_text.trim().to_owned(),
            min_similarity: self.min_similarity,
            limit: self.limit,
            cursor: None,
            source_ids: parse_source_ids(&self.source_ids)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct SemanticEvaluateForm {
    pub alert_rule_id: String,
    #[serde(default = "default_similarity")]
    pub threshold: f32,
    pub query_text: String,
    #[serde(default = "default_similarity")]
    pub min_similarity: f32,
    #[serde(default = "default_limit")]
    pub limit: u16,
    #[serde(default)]
    pub source_ids: String,
}

impl SemanticEvaluateForm {
    pub fn into_payload(self) -> Result<SemanticEvaluatePayload> {
        let alert_rule_id = self
            .alert_rule_id
            .trim()
            .parse::<Uuid>()
            .context("alert_rule_id must be a UUID")?;
        validate_score("threshold", self.threshold)?;
        let query = SemanticSearchForm {
            query_text: self.query_text,
            min_similarity: self.min_similarity,
            limit: self.limit,
            source_ids: self.source_ids,
        }
        .into_payload()?;
        Ok(SemanticEvaluatePayload {
            alert_rule_id,
            threshold: self.threshold,
            query,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticSearchPayload {
    pub query_text: String,
    pub min_similarity: f32,
    pub limit: u16,
    pub cursor: Option<String>,
    pub source_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticEvaluatePayload {
    pub alert_rule_id: Uuid,
    pub threshold: f32,
    pub query: SemanticSearchPayload,
}

fn validate_query(query: &str) -> Result<()> {
    let length = query.trim().chars().count();
    if !(3..=MAX_QUERY_CHARS).contains(&length) {
        bail!("query_text must contain 3 to {MAX_QUERY_CHARS} characters");
    }
    Ok(())
}

fn validate_score(name: &str, value: f32) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        bail!("{name} must be a finite number between 0 and 1");
    }
    Ok(())
}

fn parse_source_ids(input: &str) -> Result<Vec<Uuid>> {
    let mut ids = BTreeSet::new();
    for token in input
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
    {
        ids.insert(
            token
                .parse::<Uuid>()
                .with_context(|| format!("invalid source UUID {token:?}"))?,
        );
        if ids.len() > MAX_SOURCE_FILTERS {
            bail!("source_ids must contain at most {MAX_SOURCE_FILTERS} unique IDs");
        }
    }
    Ok(ids.into_iter().collect())
}

const fn default_similarity() -> f32 {
    0.78
}

const fn default_limit() -> u16 {
    50
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_policy_rejects_server_owned_tenant() {
        let form = SourcePolicyForm {
            policy_json: r#"{"tenant_id":"00000000-0000-0000-0000-000000000000"}"#.into(),
        };
        assert!(form.into_policy().is_err());
    }

    #[test]
    fn source_ids_are_deduplicated() {
        let id = Uuid::nil();
        let parsed = parse_source_ids(&format!("{id}, {id}")).expect("source IDs");
        assert_eq!(parsed, vec![id]);
    }

    #[test]
    fn semantic_query_is_bounded() {
        let form = SemanticSearchForm {
            query_text: "x".into(),
            min_similarity: 0.78,
            limit: 50,
            source_ids: String::new(),
        };
        assert!(form.into_payload().is_err());
    }
}
