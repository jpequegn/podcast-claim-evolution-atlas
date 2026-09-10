use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub adapter: String,
    pub database_sha256: String,
    pub episode_ids: Vec<u64>,
    pub segment_ids: Vec<u64>,
    pub query_sha256: String,
    pub record_count: usize,
    pub evidence_digest: String,
}
pub fn import(manifest_json: &str, jsonl: &str, claims_json: &str, title: &str) -> Result<Bundle> {
    require(
        manifest_json.len() <= 100_000
            && jsonl.len() <= 2_000_000
            && claims_json.len() <= 2_000_000,
        "import too large",
    )?;
    let m: Manifest = crate::strict::parse(manifest_json)?;
    require(
        m.version == 1 && m.adapter == "castflow-readonly-v1" && m.record_count <= 1000,
        "unsupported export",
    )?;
    let mh = hash(&m);
    let mut evidence = Vec::new();
    for line in jsonl.lines() {
        let e: Evidence = crate::strict::parse(line)?;
        require(e.export_digest == mh, "export provenance mismatch")?;
        require(
            e.episode_id
                .parse::<u64>()
                .is_ok_and(|id| m.episode_ids.contains(&id))
                && e.segment_id
                    .parse::<u64>()
                    .is_ok_and(|id| m.segment_ids.contains(&id)),
            "evidence not selected in manifest",
        )?;
        evidence.push(e);
    }
    require(
        evidence.len() == m.record_count,
        "export record count mismatch",
    )?;
    let mut payload = evidence.clone();
    for e in &mut payload {
        e.export_digest.clear();
    }
    require(
        hash(&payload) == m.evidence_digest,
        "export evidence content changed",
    )?;
    let claims: Vec<Claim> = crate::strict::parse(claims_json)?;
    let b = Bundle {
        version: 1,
        title: title.into(),
        evidence,
        claims,
        aliases: BTreeMap::new(),
    };
    b.validate()?;
    Ok(b)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_bad_manifest() {
        assert!(import("{}", "", "[]", "Test").is_err());
    }
    #[test]
    fn rejects_count_mismatch() {
        let m = Manifest {
            version: 1,
            adapter: "castflow-readonly-v1".into(),
            database_sha256: "a".repeat(64),
            episode_ids: vec![1],
            segment_ids: vec![1],
            query_sha256: "a".repeat(64),
            record_count: 1,
            evidence_digest: "a".repeat(64),
        };
        assert!(import(&serde_json::to_string(&m).unwrap(), "", "[]", "Test").is_err());
    }
}
