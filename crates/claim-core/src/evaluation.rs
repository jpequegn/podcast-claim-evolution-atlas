use crate::{
    graph::Document,
    rules::{classify, Relation},
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldPair {
    pub left: String,
    pub right: String,
    pub expected: Relation,
}
#[derive(Debug, Serialize)]
pub struct Evaluation {
    pub cases: usize,
    pub correct: usize,
    pub abstained: usize,
    pub false_contradictions: usize,
    pub confusion: BTreeMap<Relation, BTreeMap<Relation, usize>>,
    pub per_class: BTreeMap<Relation, ClassMetrics>,
    pub macro_f1: f64,
    pub evidence_records: usize,
    pub timestamped_records: usize,
    pub synthetic_records: usize,
    pub recorded_review_seconds: u64,
    pub timed_reviews: usize,
    pub total_reviews: usize,
    pub extraction_quality: Option<f64>,
    pub human_agreement: Option<f64>,
    pub calibration: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct ClassMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}
pub fn evaluate(d: &Document, gold: &[GoldPair]) -> Result<Evaluation> {
    d.graph()?;
    require(!gold.is_empty() && gold.len() <= 2000, "invalid gold size")?;
    let claims: BTreeMap<_, _> = d.bundle.claims.iter().map(|c| (&c.id, c)).collect();
    let mut confusion: BTreeMap<Relation, BTreeMap<Relation, usize>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut correct = 0;
    let mut abstained = 0;
    let mut false_contradictions = 0;
    for pair in gold {
        require(
            pair.left != pair.right && seen.insert((&pair.left, &pair.right)),
            "duplicate or self gold pair",
        )?;
        let a = claims
            .get(&pair.left)
            .ok_or(Error("unknown gold claim".into()))?;
        let b = claims
            .get(&pair.right)
            .ok_or(Error("unknown gold claim".into()))?;
        require(
            a.subject == b.subject && a.predicate == b.predicate && a.topic == b.topic,
            "gold pair outside candidate block",
        )?;
        let actual = classify(a, b).0;
        *confusion
            .entry(pair.expected)
            .or_default()
            .entry(actual)
            .or_default() += 1;
        correct += usize::from(actual == pair.expected);
        abstained += usize::from(actual == Relation::InsufficientEvidence);
        false_contradictions +=
            usize::from(actual == Relation::Contradicts && pair.expected != Relation::Contradicts);
    }
    let labels: BTreeSet<_> = confusion
        .keys()
        .copied()
        .chain(confusion.values().flat_map(|r| r.keys().copied()))
        .collect();
    let mut per_class = BTreeMap::new();
    for label in labels {
        let tp = *confusion
            .get(&label)
            .and_then(|r| r.get(&label))
            .unwrap_or(&0) as f64;
        let predictions = confusion
            .values()
            .map(|r| r.get(&label).copied().unwrap_or(0))
            .sum::<usize>() as f64;
        let support = confusion
            .get(&label)
            .map(|r| r.values().sum::<usize>())
            .unwrap_or(0) as f64;
        let precision = if predictions > 0.0 {
            tp / predictions
        } else {
            0.0
        };
        let recall = if support > 0.0 { tp / support } else { 0.0 };
        let f1 = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };
        per_class.insert(
            label,
            ClassMetrics {
                precision,
                recall,
                f1,
            },
        );
    }
    let macro_f1 = per_class.values().map(|v| v.f1).sum::<f64>() / per_class.len() as f64;
    Ok(Evaluation {
        cases: gold.len(),
        correct,
        abstained,
        false_contradictions,
        confusion,
        per_class,
        macro_f1,
        evidence_records: d.bundle.evidence.len(),
        timestamped_records: d
            .bundle
            .evidence
            .iter()
            .filter(|e| e.kind == EvidenceKind::Transcript)
            .count(),
        synthetic_records: d
            .bundle
            .evidence
            .iter()
            .filter(|e| e.kind == EvidenceKind::Synthetic)
            .count(),
        recorded_review_seconds: d
            .reviews
            .iter()
            .filter_map(|r| r.request.elapsed_seconds)
            .map(u64::from)
            .sum(),
        timed_reviews: d
            .reviews
            .iter()
            .filter(|r| r.request.elapsed_seconds.is_some())
            .count(),
        total_reviews: d.reviews.len(),
        extraction_quality: None,
        human_agreement: None,
        calibration: None,
    })
}
