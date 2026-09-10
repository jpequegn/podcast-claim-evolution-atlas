use crate::{
    rules::{proposals, Proposal, Relation},
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    ApproveClaim,
    RejectClaim,
    AcceptRelation,
    RejectRelation,
    Resolve,
    Supersede,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequest {
    pub revision: usize,
    pub action: Action,
    pub target: String,
    pub relation: Option<Relation>,
    pub reviewer: String,
    pub note: String,
    pub evidence_ids: Vec<String>,
    pub at: String,
    pub elapsed_seconds: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub request: ReviewRequest,
    pub bundle_digest: String,
    pub previous: String,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub bundle: Bundle,
    pub reviews: Vec<Review>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub proposal: Proposal,
    pub relation: Relation,
    pub status: String,
    pub reviewer: Option<String>,
    pub closing_evidence: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    pub claims: BTreeMap<String, String>,
    pub edges: Vec<Edge>,
    pub revision: usize,
    pub bundle_digest: String,
}
impl Document {
    pub fn new(bundle: Bundle) -> Self {
        Self {
            bundle,
            reviews: vec![],
        }
    }
    pub fn parse(input: &str) -> Result<Self> {
        require(input.len() <= 4_000_000, "document exceeds 4 MB")?;
        let d: Self = crate::strict::parse(input)?;
        d.graph()?;
        Ok(d)
    }
    pub fn graph(&self) -> Result<Graph> {
        self.bundle.validate()?;
        require(self.reviews.len() <= 2000, "too many review records")?;
        let bh = hash(&self.bundle);
        let mut g = Graph {
            claims: self
                .bundle
                .claims
                .iter()
                .map(|c| (c.id.clone(), "candidate".into()))
                .collect(),
            edges: proposals(&self.bundle)?
                .into_iter()
                .map(|p| Edge {
                    relation: p.relation,
                    proposal: p,
                    status: "candidate".into(),
                    reviewer: None,
                    closing_evidence: vec![],
                })
                .collect(),
            revision: 0,
            bundle_digest: bh.clone(),
        };
        let mut previous = String::new();
        let mut last_day = String::new();
        for review in &self.reviews {
            require(
                review.bundle_digest == bh
                    && review.previous == previous
                    && review.digest == hash(&(&review.request, &bh, &previous)),
                "review chain or bundle digest mismatch",
            )?;
            require(review.request.at >= last_day, "review time regressed")?;
            self.apply(&mut g, &review.request)?;
            previous = review.digest.clone();
            last_day = review.request.at.clone();
        }
        Ok(g)
    }
    fn apply(&self, g: &mut Graph, r: &ReviewRequest) -> Result<()> {
        require(r.revision == g.revision, "stale review revision")?;
        require(
            ident(&r.reviewer) && text(&r.note, 1000),
            "reviewer and reason required",
        )?;
        date(&r.at)?;
        require(
            r.evidence_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == r.evidence_ids.len(),
            "duplicate review evidence",
        )?;
        require(
            self.bundle
                .evidence
                .iter()
                .filter(|e| r.evidence_ids.contains(&e.id))
                .all(|e| e.asserted_at <= r.at),
            "review predates evidence",
        )?;
        require(
            r.elapsed_seconds.is_none_or(|s| s <= 86400),
            "review time out of bounds",
        )?;
        require(
            !r.evidence_ids.is_empty()
                && r.evidence_ids.len() <= 8
                && r.evidence_ids
                    .iter()
                    .all(|id| self.bundle.evidence.iter().any(|e| e.id == *id)),
            "review evidence missing",
        )?;
        if matches!(r.action, Action::ApproveClaim | Action::RejectClaim) {
            require(r.relation.is_none(), "claim review cannot set relation")?;
            let c = self
                .bundle
                .claims
                .iter()
                .find(|c| c.id == r.target)
                .ok_or(Error("unknown claim".into()))?;
            require(
                r.evidence_ids.iter().all(|id| c.evidence_ids.contains(id)),
                "claim review evidence must support target",
            )?;
            require(
                g.claims[&r.target] == "candidate",
                "claim already reviewed; import corrected claim as new version",
            )?;
            g.claims.insert(
                r.target.clone(),
                if r.action == Action::ApproveClaim {
                    "approved"
                } else {
                    "rejected"
                }
                .into(),
            );
        } else {
            let edge = g
                .edges
                .iter_mut()
                .find(|e| e.proposal.id == r.target)
                .ok_or(Error("unknown relationship".into()))?;
            match r.action {
                Action::AcceptRelation | Action::RejectRelation => {
                    require(edge.status == "candidate", "relationship already reviewed")?;
                    require(
                        r.evidence_ids
                            .iter()
                            .all(|id| edge.proposal.evidence_ids.contains(id)),
                        "review evidence must belong to pair",
                    )?;
                    if r.action == Action::AcceptRelation {
                        require(
                            g.claims[&edge.proposal.left] == "approved"
                                && g.claims[&edge.proposal.right] == "approved",
                            "approve both claims before relationship",
                        )?;
                        let relation = r.relation.unwrap_or(edge.relation);
                        require(
                            !matches!(
                                relation,
                                Relation::Resolves
                                    | Relation::Supersedes
                                    | Relation::InsufficientEvidence
                            ),
                            "closure needs its own transition; unknown is not a durable relation",
                        )?;
                        edge.relation = relation;
                        edge.status = "accepted".into();
                    } else {
                        require(r.relation.is_none(), "rejection cannot classify")?;
                        edge.status = "rejected".into();
                    }
                }
                Action::Resolve | Action::Supersede => {
                    require(
                        edge.status == "accepted" && edge.relation == Relation::Contradicts,
                        "only an accepted disagreement can close",
                    )?;
                    require(r.relation.is_none(), "closure relation is fixed")?;
                    require(
                        r.evidence_ids
                            .iter()
                            .any(|id| !edge.proposal.evidence_ids.contains(id)),
                        "closure requires additional evidence beyond the original pair",
                    )?;
                    edge.status = if r.action == Action::Resolve {
                        "resolved"
                    } else {
                        "superseded"
                    }
                    .into();
                    edge.closing_evidence = r.evidence_ids.clone();
                }
                _ => unreachable!(),
            }
            edge.reviewer = Some(r.reviewer.clone());
        }
        g.revision += 1;
        Ok(())
    }
    pub fn review(&self, r: ReviewRequest) -> Result<Self> {
        let mut g = self.graph()?;
        self.apply(&mut g, &r)?;
        let mut d = self.clone();
        let previous = d
            .reviews
            .last()
            .map(|r| r.digest.clone())
            .unwrap_or_default();
        let bundle_digest = hash(&d.bundle);
        let digest = hash(&(&r, &bundle_digest, &previous));
        d.reviews.push(Review {
            request: r,
            bundle_digest,
            previous,
            digest,
        });
        d.graph()?;
        Ok(d)
    }
    pub fn timeline(&self, subject: &str) -> Result<Vec<&Claim>> {
        self.graph()?;
        let canonical = |s: &str| {
            self.bundle
                .aliases
                .get(s)
                .map(String::as_str)
                .unwrap_or(s)
                .to_string()
        };
        let mut claims: Vec<_> = self
            .bundle
            .claims
            .iter()
            .filter(|c| canonical(&c.subject) == canonical(subject))
            .collect();
        claims.sort_by_key(|c| {
            let day = c
                .evidence_ids
                .iter()
                .filter_map(|id| self.bundle.evidence.iter().find(|e| e.id == *id))
                .map(|e| e.asserted_at.as_str())
                .min()
                .unwrap_or("");
            (day, c.id.as_str())
        });
        Ok(claims)
    }
}
