use claim_core::{graph::*, rules::*, *};
fn bundle() -> Bundle {
    Bundle::parse(include_str!("../../../examples/basic.json")).unwrap()
}
#[test]
fn duplicate_content_and_nested_alias_keys_fail() {
    let mut b = bundle();
    let mut c = b.claims[0].clone();
    c.id = "duplicate".into();
    b.claims.push(c);
    assert!(b.validate().is_err());
    let raw = serde_json::to_string(&bundle()).unwrap().replace(
        "\"aliases\":{}",
        "\"aliases\":{\"x\":\"task\",\"x\":\"other\"}",
    );
    assert!(Bundle::parse(&raw).is_err());
}
#[test]
fn changed_qualifiers_never_force_contradiction() {
    let b = bundle();
    let a = &b.claims[0];
    for mutation in 0..6 {
        let mut c = b.claims[1].clone();
        match mutation {
            0 => c.scope.population = "other".into(),
            1 => c.scope.system = "other".into(),
            2 => {
                c.scope.assumptions.insert("different".into());
            }
            3 => c.stance = Stance::Hypothetical,
            4 => c.scope.valid_from = "2026-02-01".into(),
            _ => c.modality = Modality::Recommendation,
        };
        assert_eq!(classify(a, &c).0, Relation::InsufficientEvidence);
    }
}
#[test]
fn review_cannot_predate_evidence_or_duplicate_citations() {
    let d = Document::new(bundle());
    let r = ReviewRequest {
        revision: 0,
        action: Action::ApproveClaim,
        target: "claim-1".into(),
        relation: None,
        reviewer: "operator".into(),
        note: "Checked".into(),
        evidence_ids: vec!["ev-1".into()],
        at: "2025-01-01".into(),
        elapsed_seconds: None,
    };
    assert!(d.review(r.clone()).is_err());
    let mut r = r;
    r.at = "2026-09-10".into();
    r.evidence_ids.push("ev-1".into());
    assert!(d.review(r).is_err());
}
#[test]
fn unsafe_uri_and_untimed_evidence_are_explicit() {
    let mut b = bundle();
    b.evidence[0].uri = "javascript:alert(1)".into();
    assert!(b.validate().is_err());
    let mut b = bundle();
    b.evidence[0].kind = EvidenceKind::TranscriptUntimed;
    assert!(b.validate().is_err());
    b.evidence[0].start_ms = None;
    b.evidence[0].end_ms = None;
    assert!(b.validate().is_ok());
}
