//! Compliance rule engine.
//!
//! Each spec module exposes a set of rules. A rule inspects the assembled
//! `InspectReport` and returns a pass/warn/fail with the offending
//! measurement and a citation string the UI can show in a tooltip. Rules
//! are intentionally small and pure — easy to test, easy to extend.

pub mod apple_hls;
pub mod ebu_r128;
pub mod netflix;

use serde::Serialize;

use crate::InspectReport;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
    NotApplicable,
}

#[derive(Debug, Serialize, Clone)]
pub struct RuleResult {
    pub spec: &'static str,
    pub rule: &'static str,
    pub verdict: Verdict,
    pub detail: String,
    /// Where in the UI to focus to see the supporting evidence.
    pub evidence_view: &'static str,
    /// Spec citation (e.g., "ITU-R BS.1770-4 §3.2", "Apple HLS Authoring §1.5").
    pub citation: &'static str,
}

pub fn run_all(report: &InspectReport) -> Vec<RuleResult> {
    let mut out = Vec::new();
    out.extend(apple_hls::evaluate(report));
    out.extend(ebu_r128::evaluate(report));
    out.extend(netflix::evaluate(report));
    out
}
