use crate::{graph::Document, rules::Relation, *};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Serialize)]
pub struct Rank {
    pub edge_id: String,
    pub focus_match: bool,
    pub independent_source_labels: usize,
    pub transcript_evidence: usize,
    pub age_days: i64,
    pub score: i64,
    pub status: String,
}
pub fn rank(d: &Document, as_of: &str, focus: &str) -> Result<Vec<Rank>> {
    let day = date(as_of)?;
    let g = d.graph()?;
    let mut rows = Vec::new();
    for e in g.edges.iter().filter(|e| {
        e.relation == Relation::Contradicts
            && e.status != "resolved"
            && e.status != "superseded"
            && e.status != "rejected"
    }) {
        let evidence: Vec<_> = d
            .bundle
            .evidence
            .iter()
            .filter(|v| e.proposal.evidence_ids.contains(&v.id))
            .collect();
        let newest = evidence
            .iter()
            .map(|v| date(&v.asserted_at))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .ok_or(Error("no evidence".into()))?;
        require(newest <= day, "report as-of predates source evidence")?;
        let age = (day - newest).whole_days();
        let sources = evidence
            .iter()
            .map(|v| &v.source)
            .collect::<BTreeSet<_>>()
            .len();
        let transcripts = evidence
            .iter()
            .filter(|v| v.kind == EvidenceKind::Transcript)
            .count();
        let matched = d
            .bundle
            .claims
            .iter()
            .any(|c| c.id == e.proposal.left && c.topic == focus);
        let score = i64::from(matched) * 30
            + (sources.min(3) * 10) as i64
            + (transcripts.min(2) * 10) as i64
            + (30 - age).max(0);
        rows.push(Rank {
            edge_id: e.proposal.id.clone(),
            focus_match: matched,
            independent_source_labels: sources,
            transcript_evidence: transcripts,
            age_days: age,
            score,
            status: e.status.clone(),
        });
    }
    rows.sort_by(|a, b| b.score.cmp(&a.score).then(a.edge_id.cmp(&b.edge_id)));
    Ok(rows)
}
#[derive(Debug, Serialize)]
pub struct Change {
    pub id: String,
    pub kind: String,
}
pub fn diff(a: &Document, b: &Document) -> Result<Vec<Change>> {
    let (ag, bg) = (a.graph()?, b.graph()?);
    let mut out = Vec::new();
    for (id, state) in &bg.claims {
        if ag.claims.get(id) != Some(state) {
            out.push(Change {
                id: id.clone(),
                kind: format!("claim_review_{state}"),
            });
        }
    }
    let old_evidence: BTreeMap<_, _> = a.bundle.evidence.iter().map(|e| (&e.id, hash(e))).collect();
    for e in &b.bundle.evidence {
        if old_evidence.get(&e.id) != Some(&hash(e)) {
            out.push(Change {
                id: e.id.clone(),
                kind: "evidence_added_or_changed".into(),
            });
        }
    }
    let ac: BTreeMap<_, _> = a.bundle.claims.iter().map(|c| (&c.id, hash(c))).collect();
    let bc: BTreeMap<_, _> = b.bundle.claims.iter().map(|c| (&c.id, hash(c))).collect();
    for (id, h) in &bc {
        if ac.get(id) != Some(h) {
            out.push(Change {
                id: (*id).clone(),
                kind: if ac.contains_key(id) {
                    "claim_changed"
                } else {
                    "claim_added"
                }
                .into(),
            });
        }
    }
    for id in ac.keys() {
        if !bc.contains_key(id) {
            out.push(Change {
                id: (*id).clone(),
                kind: "claim_removed".into(),
            });
        }
    }
    let ae: BTreeMap<_, _> = ag.edges.iter().map(|e| (&e.proposal.id, hash(e))).collect();
    for e in &bg.edges {
        if ae.get(&e.proposal.id) != Some(&hash(e)) {
            out.push(Change {
                id: e.proposal.id.clone(),
                kind: format!("relationship_{}", e.status),
            });
        }
    }
    for e in &ag.edges {
        if !bg.edges.iter().any(|x| x.proposal.id == e.proposal.id) {
            out.push(Change {
                id: e.proposal.id.clone(),
                kind: "relationship_removed".into(),
            });
        }
    }
    Ok(out)
}
fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace(['\n', '\r'], " ")
}
pub fn markdown(
    d: &Document,
    as_of: &str,
    focus: &str,
    previous: Option<&Document>,
) -> Result<String> {
    let g = d.graph()?;
    let ranked = rank(d, as_of, focus)?;
    let mut out=format!("# Claim evolution report\n\nAs of {as_of}. Bundle {}. Review revision {}.\n\n{} claims, {} candidate pairs. Proposals are not reviewed conclusions.\n\n## Unresolved disagreements\n\n| Pair | Status | Score | Focus | Source labels | Transcript evidence | Age days |\n| --- | --- | ---: | --- | ---: | ---: | ---: |\n",g.bundle_digest,g.revision,d.bundle.claims.len(),g.edges.len());
    for r in ranked {
        let e = g
            .edges
            .iter()
            .find(|e| e.proposal.id == r.edge_id)
            .expect("ranked edge");
        out.push_str(&format!(
            "| {} / {} | {} | {} | {} | {} | {} | {} |\n",
            cell(&e.proposal.left),
            cell(&e.proposal.right),
            r.status,
            r.score,
            r.focus_match,
            r.independent_source_labels,
            r.transcript_evidence,
            r.age_days
        ));
    }
    out.push_str("\nScores prioritize review only. Source labels do not establish independence or truth.\n\n## New and reframed\n\n");
    if let Some(p) = previous {
        for c in diff(p, d)? {
            out.push_str(&format!("- {}: {}\n", c.kind, cell(&c.id)));
        }
    } else {
        out.push_str("No baseline supplied; growth and acceleration are not inferred.\n");
    }
    out.push_str("\n## Resolved or superseded\n\n");
    for e in g
        .edges
        .iter()
        .filter(|e| matches!(e.status.as_str(), "resolved" | "superseded"))
    {
        out.push_str(&format!(
            "- {}: {}. Closing evidence: {}\n",
            e.proposal.id,
            e.status,
            e.closing_evidence.join(", ")
        ));
    }
    out.push_str("\n## Evidence index\n\n");
    for e in &d.bundle.evidence {
        out.push_str(&format!(
            "- {}: {}, {}, segment {}, {:?} ms. {}\n",
            cell(&e.id),
            cell(&e.source),
            e.asserted_at,
            cell(&e.segment_id),
            e.start_ms,
            cell(&e.uri)
        ));
    }
    Ok(out)
}
