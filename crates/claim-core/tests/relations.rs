use claim_core::{rules::*, *};
fn bundle() -> Bundle {
    Bundle::parse(include_str!("../../../examples/basic.json")).unwrap()
}
#[test]
fn scope_and_stance_guards() {
    let b = bundle();
    let a = &b.claims[0];
    let mut z = b.claims[1].clone();
    assert_eq!(classify(a, &z).0, Relation::Contradicts);
    z.scope.population = "different".into();
    assert_eq!(classify(a, &z).1, "scope_mismatch");
    z = a.clone();
    z.stance = Stance::Quoted;
    assert_eq!(classify(a, &z).1, "non_asserted_stance");
    z = a.clone();
    z.scope.definition = "other".into();
    assert_eq!(classify(a, &z).0, Relation::DefinitionMismatch);
    z = a.clone();
    z.scope.valid_from = "2027-01-01".into();
    z.scope.valid_to = "2027-12-31".into();
    assert_eq!(classify(a, &z).0, Relation::TemporalChange);
}
#[test]
fn refinement_and_unknowns() {
    let b = bundle();
    let mut a = b.claims[0].clone();
    let mut z = a.clone();
    z.scope.conditions.insert("warm-start".into());
    assert_eq!(classify(&a, &z).0, Relation::Narrows);
    a.value = Value::Number {
        decimal: "1".into(),
        unit: "s".into(),
        comparator: Comparator::Lt,
    };
    z = a.clone();
    z.value = Value::Number {
        decimal: "500".into(),
        unit: "ms".into(),
        comparator: Comparator::Lt,
    };
    assert_eq!(classify(&a, &z).0, Relation::Refines);
    z.scope.population = "unknown".into();
    assert_eq!(classify(&a, &z).1, "unknown_scope");
}
#[test]
fn proposal_order_independent() {
    let mut b = bundle();
    let p = proposals(&b).unwrap();
    b.claims.reverse();
    b.evidence.reverse();
    assert_eq!(p, proposals(&b).unwrap());
}
