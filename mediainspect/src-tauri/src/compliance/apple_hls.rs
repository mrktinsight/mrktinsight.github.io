//! Apple HLS Authoring Spec — single-file checks.
//!
//! True manifest/segment conformance (IDR alignment across renditions,
//! segment-duration drift) is Phase 2 and will live in
//! `probe::manifest`. The rules here are the subset that can be
//! evaluated from a single video file's stream parameters.

use super::{RuleResult, Verdict};
use crate::InspectReport;

pub fn evaluate(report: &InspectReport) -> Vec<RuleResult> {
    let mut out = Vec::new();
    let Some(ff) = &report.ffprobe else { return out; };

    let video = ff
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|streams| streams.iter().find(|s| s["codec_type"] == "video"));

    let Some(v) = video else { return out; };

    let codec = v.get("codec_name").and_then(|x| x.as_str()).unwrap_or("");
    let codec_ok = matches!(codec, "h264" | "hevc");
    out.push(RuleResult {
        spec: "Apple HLS",
        rule: "Video codec (H.264 or HEVC)",
        verdict: if codec_ok { Verdict::Pass } else { Verdict::Fail },
        detail: format!("codec = {codec}"),
        evidence_view: "streams",
        citation: "Apple HLS Authoring §1.4",
    });

    let pix_fmt = v.get("pix_fmt").and_then(|x| x.as_str()).unwrap_or("");
    let yuv420 = pix_fmt.contains("yuv420");
    out.push(RuleResult {
        spec: "Apple HLS",
        rule: "4:2:0 chroma subsampling for SDR",
        verdict: if yuv420 { Verdict::Pass } else { Verdict::Warn },
        detail: format!("pix_fmt = {pix_fmt}"),
        evidence_view: "streams",
        citation: "Apple HLS Authoring §2.1",
    });

    // Max keyframe interval ≤ 6 s (informational from single-file stats).
    if let Some(fps) = v.get("avg_frame_rate").and_then(|x| x.as_str()) {
        if let Some(rate) = parse_rational(fps) {
            let max_keyint_seconds = 6.0;
            let max_keyint_frames = (rate * max_keyint_seconds).round() as i64;
            out.push(RuleResult {
                spec: "Apple HLS",
                rule: "GOP duration ≤ 6 s",
                verdict: Verdict::Warn, // can't prove from single-file ffprobe; flag for review
                detail: format!(
                    "fps ≈ {rate:.3}; recommended max keyint ≈ {max_keyint_frames} frames \
                     (verify with GOP timeline)"
                ),
                evidence_view: "timeline",
                citation: "Apple HLS Authoring §4.10",
            });
        }
    }

    let profile = v.get("profile").and_then(|x| x.as_str()).unwrap_or("");
    out.push(RuleResult {
        spec: "Apple HLS",
        rule: "Codec profile declared",
        verdict: if !profile.is_empty() { Verdict::Pass } else { Verdict::Warn },
        detail: format!("profile = {profile}"),
        evidence_view: "streams",
        citation: "Apple HLS Authoring §1.4.1",
    });

    out
}

fn parse_rational(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/')?;
    let n: f64 = n.parse().ok()?;
    let d: f64 = d.parse().ok()?;
    if d == 0.0 { None } else { Some(n / d) }
}
