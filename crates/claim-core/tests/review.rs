use claim_core::{graph::*, *};
fn doc() -> Document {
    Document::new(Bundle::parse(include_str!("../../../examples/basic.json")).unwrap())
}
fn request(revision: usize, action: Action, target: &str, ids: &[&str]) -> ReviewRequest {
    ReviewRequest {
        revision,
        action,
        target: target.into(),
        relation: None,
        reviewer: "operator".into(),
        note: "Checked evidence and applicable scope".into(),
        evidence_ids: ids.iter().map(|s| s.to_string()).collect(),
        at: "2026-09-10".into(),
        elapsed_seconds: Some(30),
    }
}
#[test]
fn durable_review_requires_claim_approval() {
    let d = doc();
    let id = d.graph().unwrap().edges[0].proposal.id.clone();
    assert!(d
        .review(request(0, Action::AcceptRelation, &id, &["ev-1"]))
        .is_err());
    let d = d
        .review(request(0, Action::ApproveClaim, "claim-1", &["ev-1"]))
        .unwrap();
    assert!(d
        .review(request(0, Action::ApproveClaim, "claim-2", &["ev-2"]))
        .is_err());
    let d = d
        .review(request(1, Action::ApproveClaim, "claim-2", &["ev-2"]))
        .unwrap();
    let d = d
        .review(request(2, Action::AcceptRelation, &id, &["ev-1", "ev-2"]))
        .unwrap();
    assert_eq!(d.graph().unwrap().edges[0].status, "accepted");
    assert!(d
        .review(request(3, Action::Resolve, &id, &["ev-1"]))
        .is_err());
}
#[test]
fn closure_new_evidence_and_no_silent_resolution() {
    let mut d = doc();
    let mut extra = d.bundle.evidence[0].clone();
    extra.id = "later-evidence".into();
    d.bundle.evidence.push(extra);
    let id = d.graph().unwrap().edges[0].proposal.id.clone();
    for r in [
        request(0, Action::ApproveClaim, "claim-1", &["ev-1"]),
        request(1, Action::ApproveClaim, "claim-2", &["ev-2"]),
        request(2, Action::AcceptRelation, &id, &["ev-1", "ev-2"]),
    ] {
        d = d.review(r).unwrap();
    }
    assert_eq!(d.graph().unwrap(), d.graph().unwrap());
    let d = d
        .review(request(3, Action::Resolve, &id, &["later-evidence"]))
        .unwrap();
    assert_eq!(d.graph().unwrap().edges[0].status, "resolved");
    assert!(d
        .review(request(4, Action::Resolve, &id, &["later-evidence"]))
        .is_err());
}
#[test]
fn chain_detects_mutation_and_source_change() {
    let d = doc()
        .review(request(0, Action::ApproveClaim, "claim-1", &["ev-1"]))
        .unwrap();
    let mut changed = d.clone();
    changed.reviews[0].request.note = "Changed".into();
    assert!(changed.graph().is_err());
    let mut changed = d;
    changed.bundle.claims[0].statement = "Changed claim".into();
    assert!(changed.graph().is_err());
}
#[test]
fn timeline_is_ordered_and_unknown_subject_empty() {
    let d = doc();
    assert_eq!(d.timeline("task").unwrap().len(), 2);
    assert!(d.timeline("absent").unwrap().is_empty());
}
