use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Relation {
    Repeats,
    Supports,
    Contradicts,
    Refines,
    Narrows,
    Broadens,
    Supersedes,
    TemporalChange,
    DefinitionMismatch,
    InsufficientEvidence,
    Resolves,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub id: String,
    pub left: String,
    pub right: String,
    pub relation: Relation,
    pub reason: String,
    pub rule: String,
    pub evidence_ids: Vec<String>,
}

// Six decimal places, scaled to exact millionths. Deliberately no exponent/float parsing.
pub fn decimal(value: &str) -> Result<i128> {
    let (sign, raw) = if let Some(v) = value.strip_prefix('-') {
        (-1, v)
    } else {
        (1, value)
    };
    let parts: Vec<_> = raw.split('.').collect();
    require(
        parts.len() <= 2
            && !parts[0].is_empty()
            && parts[0].len() <= 12
            && parts[0].bytes().all(|b| b.is_ascii_digit()),
        "invalid exact decimal",
    )?;
    let frac = parts.get(1).copied().unwrap_or("");
    require(
        frac.len() <= 6
            && frac.bytes().all(|b| b.is_ascii_digit())
            && (parts.len() == 1 || !frac.is_empty()),
        "decimal supports up to six places",
    )?;
    let whole = parts[0]
        .parse::<i128>()
        .map_err(|_| Error("number out of range".into()))?;
    let f = format!("{frac:0<6}")
        .parse::<i128>()
        .map_err(|_| Error("invalid decimal".into()))?;
    Ok(sign * (whole * 1_000_000 + f))
}
fn quantity(v: &str, unit: &str) -> Result<(i128, &'static str)> {
    let n = decimal(v)?;
    // Nanounits preserve six input decimals through percentage and millisecond conversion.
    Ok(match unit {
        "ratio" => (n * 1000, "ratio"),
        "%" | "percent" => (n * 10, "ratio"),
        "ms" => (n, "seconds"),
        "s" | "seconds" => (n * 1000, "seconds"),
        "min" | "minutes" => (n * 60_000, "seconds"),
        "count" => (n * 1000, "count"),
        "usd" => (n * 1000, "usd"),
        _ => return Err(Error("unsupported unit".into())),
    })
}
#[derive(Clone, Copy)]
struct Interval {
    low: Option<(i128, bool)>,
    high: Option<(i128, bool)>,
}
fn interval(n: i128, c: &Comparator) -> Interval {
    match c {
        Comparator::Eq => Interval {
            low: Some((n, true)),
            high: Some((n, true)),
        },
        Comparator::Gt => Interval {
            low: Some((n, false)),
            high: None,
        },
        Comparator::Ge => Interval {
            low: Some((n, true)),
            high: None,
        },
        Comparator::Lt => Interval {
            low: None,
            high: Some((n, false)),
        },
        Comparator::Le => Interval {
            low: None,
            high: Some((n, true)),
        },
    }
}
fn disjoint(a: Interval, b: Interval) -> bool {
    let before = |high: Option<(i128, bool)>, low: Option<(i128, bool)>| matches!((high,low),(Some((h,hi)),Some((l,li))) if h<l || h==l && !(hi && li));
    before(a.high, b.low) || before(b.high, a.low)
}
fn subset(a: Interval, b: Interval) -> bool {
    let lower = match (a.low, b.low) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some((a, ai)), Some((b, bi))) => a > b || a == b && (!ai || bi),
    };
    let upper = match (a.high, b.high) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some((a, ai)), Some((b, bi))) => a < b || a == b && (!ai || bi),
    };
    lower && upper
}
pub fn classify(a: &Claim, b: &Claim) -> (Relation, &'static str) {
    use Relation::*;
    if a.stance != Stance::Asserted || b.stance != Stance::Asserted {
        return (InsufficientEvidence, "non_asserted_stance");
    }
    if a.modality != b.modality
        || matches!(
            a.modality,
            Modality::Hypothesis
                | Modality::Preference
                | Modality::Recommendation
                | Modality::Causal
        )
    {
        return (InsufficientEvidence, "modality_not_decidable");
    }
    let x = &a.scope;
    let y = &b.scope;
    if [
        &x.population,
        &x.geography,
        &x.system,
        &x.definition,
        &y.population,
        &y.geography,
        &y.system,
        &y.definition,
    ]
    .iter()
    .any(|s| s.as_str() == "unknown")
    {
        return (InsufficientEvidence, "unknown_scope");
    }
    if x.population != y.population
        || x.geography != y.geography
        || x.system != y.system
        || x.assumptions != y.assumptions
        || x.exclusions != y.exclusions
    {
        return (InsufficientEvidence, "scope_mismatch");
    }
    if x.definition != y.definition {
        return (DefinitionMismatch, "different_definitions");
    }
    if x.valid_to < y.valid_from || y.valid_to < x.valid_from {
        return (TemporalChange, "disjoint_valid_time");
    }
    if x.valid_from != y.valid_from || x.valid_to != y.valid_to {
        return (InsufficientEvidence, "partial_time_overlap");
    }
    if x.conditions != y.conditions {
        if a.value == b.value && a.positive == b.positive {
            if x.conditions.is_subset(&y.conditions) {
                return (Narrows, "added_conditions");
            }
            if y.conditions.is_subset(&x.conditions) {
                return (Broadens, "removed_conditions");
            }
        }
        return (InsufficientEvidence, "condition_mismatch");
    }
    match (&a.value, &b.value) {
        (Value::Boolean { value: av }, Value::Boolean { value: bv }) => {
            if (*av == a.positive) == (*bv == b.positive) {
                (Repeats, "same_boolean")
            } else {
                (Contradicts, "opposite_boolean")
            }
        }
        (
            Value::Number {
                decimal: av,
                unit: au,
                comparator: ac,
            },
            Value::Number {
                decimal: bv,
                unit: bu,
                comparator: bc,
            },
        ) if a.positive && b.positive => {
            let (Ok((an, ad)), Ok((bn, bd))) = (quantity(av, au), quantity(bv, bu)) else {
                return (InsufficientEvidence, "unsupported_quantity");
            };
            if ad != bd {
                return (InsufficientEvidence, "unit_mismatch");
            }
            let (ai, bi) = (interval(an, ac), interval(bn, bc));
            if disjoint(ai, bi) {
                (Contradicts, "disjoint_numeric_constraints")
            } else if subset(ai, bi) && subset(bi, ai) {
                (Repeats, "equivalent_quantity")
            } else if subset(bi, ai) {
                (Refines, "tighter_numeric_constraint")
            } else {
                (InsufficientEvidence, "overlapping_numeric_constraints")
            }
        }
        (Value::Text { value: av }, Value::Text { value: bv }) if av == bv => {
            if a.positive != b.positive {
                (Contradicts, "explicit_polarity_conflict")
            } else {
                (Repeats, "identical_text_value")
            }
        }
        _ => (InsufficientEvidence, "semantic_review_required"),
    }
}
pub fn proposals(b: &Bundle) -> Result<Vec<Proposal>> {
    b.validate()?;
    let canonical = |s: &String| b.aliases.get(s).unwrap_or(s).clone();
    let mut claims: Vec<_> = b.claims.iter().collect();
    // Chronological direction is stable even when input files are reordered.
    claims.sort_by_key(|c| {
        let day = c
            .evidence_ids
            .iter()
            .filter_map(|id| b.evidence.iter().find(|e| e.id == *id))
            .map(|e| e.asserted_at.as_str())
            .min()
            .unwrap_or("");
        (day, c.id.as_str())
    });
    let mut out = Vec::new();
    for (i, a) in claims.iter().enumerate() {
        for other in claims.iter().skip(i + 1) {
            let z = *other;
            if canonical(&a.subject) != canonical(&z.subject)
                || a.predicate != z.predicate
                || a.topic != z.topic
            {
                continue;
            }
            require(out.len() < 5000, "too many candidate pairs; narrow bundle")?;
            let (relation, reason) = classify(a, z);
            let evidence_ids: BTreeSet<_> = a
                .evidence_ids
                .iter()
                .chain(z.evidence_ids.iter())
                .cloned()
                .collect();
            let id = hash(&(hash(*a), hash(z), relation, reason, "rules-v1"));
            out.push(Proposal {
                id,
                left: a.id.clone(),
                right: z.id.clone(),
                relation,
                reason: reason.into(),
                rule: "rules-v1".into(),
                evidence_ids: evidence_ids.into_iter().collect(),
            });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test] fn percent_ratio(n in -10000i64..10000) {
            prop_assert_eq!(quantity(&n.to_string(),"%").unwrap().0, quantity(&(n*10).to_string(),"ratio").unwrap().0/1000);
        }
        #[test] fn interval_self(n in -100000i64..100000) {
            let i=interval(i128::from(n),&Comparator::Eq);
            prop_assert!(subset(i,i)); prop_assert!(!disjoint(i,i));
        }
    }
    #[test]
    fn boundaries() {
        assert!(!disjoint(
            interval(5, &Comparator::Le),
            interval(5, &Comparator::Ge)
        ));
        assert!(disjoint(
            interval(5, &Comparator::Lt),
            interval(5, &Comparator::Ge)
        ));
    }
    #[test]
    fn exact_parser() {
        assert_eq!(decimal("-0.000001").unwrap(), -1);
        for v in ["NaN", "1e2", "+2", "1.", "1.0000001"] {
            assert!(decimal(v).is_err());
        }
    }
}
