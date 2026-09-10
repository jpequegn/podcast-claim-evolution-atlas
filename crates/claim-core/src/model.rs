use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use time::{format_description::well_known::Iso8601, Date};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);
pub fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error(message.into()))
    }
}
pub fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(|c| c.is_control())
}
pub fn ident(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
pub fn date(value: &str) -> Result<Date> {
    require(value.len() == 10, "date must be YYYY-MM-DD")?;
    Date::parse(value, &Iso8601::DEFAULT).map_err(|_| Error("invalid date".into()))
}
pub fn hash<T: Serialize>(value: &T) -> String {
    // BTreeMap-backed JSON objects give deterministic key order. No floating values in contracts.
    let value = serde_json::to_value(value).expect("serializable contract");
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).expect("JSON value"))
    )
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Fact,
    Causal,
    Prediction,
    Recommendation,
    Definition,
    Preference,
    Hypothesis,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    Asserted,
    Quoted,
    Hypothetical,
    Unclear,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparator {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Number {
        decimal: String,
        unit: String,
        comparator: Comparator,
    },
    Boolean {
        value: bool,
    },
    Text {
        value: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub population: String,
    pub geography: String,
    pub system: String,
    pub definition: String,
    pub conditions: BTreeSet<String>,
    pub assumptions: BTreeSet<String>,
    pub exclusions: BTreeSet<String>,
    pub valid_from: String,
    pub valid_to: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Transcript,
    TranscriptUntimed,
    SummaryOnly,
    Synthetic,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: String,
    pub episode_id: String,
    pub episode_title: String,
    pub source: String,
    pub speaker: String,
    pub uri: String,
    pub asserted_at: String,
    pub kind: EvidenceKind,
    pub segment_id: String,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub excerpt: String,
    pub segment_digest: String,
    pub export_digest: String,
}
impl Evidence {
    pub fn validate(&self) -> Result<()> {
        require(
            ident(&self.id) && ident(&self.episode_id) && ident(&self.segment_id),
            "invalid evidence IDs",
        )?;
        require(
            text(&self.source, 200) && text(&self.speaker, 200) && text(&self.episode_title, 500),
            "missing attribution",
        )?;
        require(
            self.uri.len() <= 2000
                && (self.uri.starts_with("https://")
                    || self.uri.starts_with("http://")
                    || self.uri.starts_with("synthetic:")),
            "unsafe source URI",
        )?;
        date(&self.asserted_at)?;
        require(text(&self.excerpt, 800), "excerpt must be bounded")?;
        for h in [&self.segment_digest, &self.export_digest] {
            require(
                h.len() == 64
                    && h.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                "invalid evidence digest",
            )?;
        }
        match self.kind {
            EvidenceKind::Transcript => require(
                matches!((self.start_ms,self.end_ms), (Some(a),Some(b)) if a < b && b <= 86_400_000),
                "transcript needs valid timestamps",
            )?,
            EvidenceKind::SummaryOnly | EvidenceKind::TranscriptUntimed => require(
                self.start_ms.is_none() && self.end_ms.is_none(),
                "summary cannot claim timestamps",
            )?,
            EvidenceKind::Synthetic => {}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub subject: String,
    pub predicate: String,
    pub topic: String,
    pub value: Value,
    pub positive: bool,
    pub modality: Modality,
    pub stance: Stance,
    pub scope: Scope,
    pub evidence_ids: Vec<String>,
    pub extractor: String,
    pub confidence_bp: u16,
    pub sensitivity: String,
    pub retention: String,
}
impl Claim {
    pub fn validate(&self) -> Result<()> {
        require(
            [&self.id, &self.subject, &self.predicate, &self.topic]
                .iter()
                .all(|v| ident(v)),
            "invalid claim identifiers",
        )?;
        require(
            text(&self.statement, 1000) && text(&self.extractor, 100),
            "invalid statement or extractor",
        )?;
        require(self.confidence_bp <= 10000, "confidence out of bounds")?;
        require(
            matches!(self.sensitivity.as_str(), "public" | "private")
                && matches!(self.retention.as_str(), "persistent" | "session"),
            "invalid retention policy",
        )?;
        require(
            !self.evidence_ids.is_empty()
                && self.evidence_ids.len() <= 8
                && self.evidence_ids.iter().collect::<BTreeSet<_>>().len()
                    == self.evidence_ids.len(),
            "invalid evidence references",
        )?;
        require(
            date(&self.scope.valid_from)? <= date(&self.scope.valid_to)?,
            "reversed valid time",
        )?;
        require(
            [
                &self.scope.population,
                &self.scope.geography,
                &self.scope.system,
                &self.scope.definition,
            ]
            .iter()
            .all(|v| text(v, 200)),
            "scope must be explicit",
        )?;
        for set in [
            &self.scope.conditions,
            &self.scope.assumptions,
            &self.scope.exclusions,
        ] {
            require(
                set.len() <= 12 && set.iter().all(|v| text(v, 200)),
                "invalid scope qualifiers",
            )?;
        }
        match &self.value {
            Value::Number { decimal, unit, .. } => {
                require(text(unit, 20), "unit required")?;
                require(crate::rules::decimal(decimal).is_ok(), "invalid number")?;
            }
            Value::Text { value } => require(text(value, 400), "invalid text value")?,
            Value::Boolean { .. } => {}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub version: u32,
    pub title: String,
    pub evidence: Vec<Evidence>,
    pub claims: Vec<Claim>,
    pub aliases: BTreeMap<String, String>,
}
impl Bundle {
    pub fn validate(&self) -> Result<()> {
        require(self.version == 1, "unsupported version")?;
        require(
            text(&self.title, 200)
                && !self.claims.is_empty()
                && self.claims.len() <= 500
                && self.evidence.len() <= 1000
                && self.aliases.len() <= 100,
            "bundle size out of bounds",
        )?;
        let mut evidence = BTreeSet::new();
        for e in &self.evidence {
            e.validate()?;
            require(evidence.insert(&e.id), "duplicate evidence ID")?;
        }
        let mut ids = BTreeSet::new();
        for c in &self.claims {
            c.validate()?;
            require(ids.insert(&c.id), "duplicate claim ID")?;
            require(
                c.evidence_ids.iter().all(|e| evidence.contains(e)),
                "missing evidence",
            )?;
        }
        for (a, b) in &self.aliases {
            require(
                ident(a) && ident(b) && a != b && !self.aliases.contains_key(b),
                "aliases must map directly to canonical IDs",
            )?;
        }
        Ok(())
    }
    pub fn parse(input: &str) -> Result<Self> {
        require(input.len() <= 2_000_000, "bundle exceeds 2 MB")?;
        let b: Self = serde_json::from_str(input).map_err(|e| Error(e.to_string()))?;
        b.validate()?;
        Ok(b)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates_are_strict() {
        assert!(date("2026-02-29").is_err());
        assert!(date("2026-01-01").is_ok());
    }
    #[test]
    fn canonical_objects() {
        assert_eq!(
            hash(&serde_json::json!({"a":1,"b":2})),
            hash(&serde_json::json!({"b":2,"a":1}))
        );
    }
    #[test]
    fn closed_value_schema() {
        assert!(serde_json::from_str::<Value>(
            r#"{"kind":"boolean","value":true,"authority":"approved"}"#
        )
        .is_err());
    }
    #[test]
    fn rejects_version_and_oversize() {
        assert!(Bundle::parse(
            r#"{"version":2,"title":"X","evidence":[],"claims":[],"aliases":{}}"#
        )
        .is_err());
        assert!(Bundle::parse(&" ".repeat(2_000_001)).is_err());
    }
}
