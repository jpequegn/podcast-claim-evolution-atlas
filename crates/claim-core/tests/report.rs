use claim_core::{graph::Document, report::*, *};
#[test]
fn ranking_and_baseline_are_explicit() {
    let d = Document::new(Bundle::parse(include_str!("../../../examples/basic.json")).unwrap());
    assert_eq!(rank(&d, "2026-09-10", "agent-harness").unwrap().len(), 1);
    assert!(rank(&d, "2025-01-01", "agent-harness").is_err());
    assert!(markdown(&d, "2026-09-10", "agent-harness", None)
        .unwrap()
        .contains("No baseline supplied"));
    assert!(diff(&d, &d).unwrap().is_empty());
}
