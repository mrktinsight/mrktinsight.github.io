//! EBU R128 / ATSC A/85 loudness conformance.
//!
//! Targets:
//! - EBU R128 broadcast: integrated -23 LUFS ±1, true peak ≤ -1 dBTP.
//! - ATSC A/85 (US broadcast): integrated -24 LUFS ±2.
//! - Common streaming references (informational; not pass/fail in this set):
//!   Spotify -14, Apple Music -16, YouTube -14.

use super::{RuleResult, Verdict};
use crate::InspectReport;

pub fn evaluate(report: &InspectReport) -> Vec<RuleResult> {
    let mut out = Vec::new();
    let Some(l) = &report.loudness else {
        return vec![RuleResult {
            spec: "EBU R128",
            rule: "Loudness measured",
            verdict: Verdict::NotApplicable,
            detail: "No decodable audio track was measured.".into(),
            evidence_view: "loudness",
            citation: "EBU R128 §3",
        }];
    };

    // Integrated within -23 ±1 LUFS.
    let integrated = l.integrated_lufs;
    let diff = (integrated - (-23.0)).abs();
    out.push(RuleResult {
        spec: "EBU R128",
        rule: "Integrated loudness target -23 LUFS ±1",
        verdict: if diff <= 1.0 {
            Verdict::Pass
        } else if diff <= 2.0 {
            Verdict::Warn
        } else {
            Verdict::Fail
        },
        detail: format!("integrated = {integrated:.2} LUFS (delta {diff:.2} LU)"),
        evidence_view: "loudness",
        citation: "EBU R128 §3.1; ITU-R BS.1770-4",
    });

    // True peak <= -1 dBTP on every channel.
    let worst = l
        .true_peak_dbtp
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    out.push(RuleResult {
        spec: "EBU R128",
        rule: "True peak ≤ -1 dBTP",
        verdict: if worst <= -1.0 {
            Verdict::Pass
        } else if worst <= 0.0 {
            Verdict::Warn
        } else {
            Verdict::Fail
        },
        detail: format!("max true peak = {worst:.2} dBTP"),
        evidence_view: "loudness",
        citation: "EBU R128 s2 §3.3",
    });

    // ATSC A/85 (US broadcast) at -24 LUFS ±2.
    let atsc_diff = (integrated - (-24.0)).abs();
    out.push(RuleResult {
        spec: "ATSC A/85",
        rule: "Integrated loudness target -24 LKFS ±2",
        verdict: if atsc_diff <= 2.0 {
            Verdict::Pass
        } else {
            Verdict::Warn
        },
        detail: format!("integrated = {integrated:.2} LUFS (delta {atsc_diff:.2} LK)"),
        evidence_view: "loudness",
        citation: "ATSC A/85 §5.5",
    });

    // Streaming reference: Spotify -14, Apple -16, YouTube -14.
    for (target, name) in [(-14.0, "Spotify"), (-16.0, "Apple Music"), (-14.0, "YouTube")] {
        let d: f64 = integrated - target;
        let verdict = if d.abs() <= 1.0 {
            Verdict::Pass
        } else {
            // Streaming platforms normalize, so a loud source is just lowered;
            // this is informational, not a fail.
            Verdict::Warn
        };
        out.push(RuleResult {
            spec: "Streaming",
            rule: "Reference target",
            verdict,
            detail: format!("{name} target {target:.0} LUFS; source {integrated:.2} LUFS (Δ {d:+.2})"),
            evidence_view: "loudness",
            citation: "Platform-published delivery specs",
        });
    }

    out
}
