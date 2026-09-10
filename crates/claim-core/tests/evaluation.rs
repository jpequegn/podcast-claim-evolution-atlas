use claim_core::{evaluation::*, graph::Document, *};
#[test]
fn independent_gold_labels() {
    let d = Document::new(Bundle::parse(include_str!("../../../examples/corpus.json")).unwrap());
    let gold: Vec<GoldPair> =
        serde_json::from_str(include_str!("../../../examples/gold.json")).unwrap();
    let e = evaluate(&d, &gold).unwrap();
    assert_eq!(d.bundle.claims.len(), 60);
    assert_eq!(e.correct, 30);
    assert_eq!(e.abstained, 9);
    assert_eq!(e.false_contradictions, 0);
    assert!(e.human_agreement.is_none());
    assert_eq!(d.timeline("agent-harness-policy").unwrap().len(), 20);
    let mut wrong = gold;
    wrong[0].expected = claim_core::rules::Relation::Repeats;
    assert_eq!(evaluate(&d, &wrong).unwrap().false_contradictions, 1);
}
