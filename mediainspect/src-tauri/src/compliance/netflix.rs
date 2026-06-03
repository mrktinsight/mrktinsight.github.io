//! Netflix delivery — narrow subset evaluable from single-file probe.
//!
//! Real Netflix delivery (Photon, IMF) is a deep workflow. Here we
//! evaluate a handful of headline requirements that catch the most
//! common screw-ups when delivering a finished video master.

use super::{RuleResult, Verdict};
use crate::InspectReport;

pub fn evaluate(report: &InspectReport) -> Vec<RuleResult> {
    let mut out = Vec::new();
    let Some(ff) = &report.ffprobe else { return out; };

    let video = ff
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|streams| streams.iter().find(|s| s["codec_type"] == "video"));

    if let Some(v) = video {
        // 1080p ProRes 422 HQ or higher is a common deliverable; here we
        // just check that the color tagging is present, since "missing color
        // primaries/transfer/matrix" is the single most common delivery bug.
        let primaries = v.get("color_primaries").and_then(|x| x.as_str()).unwrap_or("");
        let transfer = v.get("color_transfer").and_then(|x| x.as_str()).unwrap_or("");
        let matrix = v.get("color_space").and_then(|x| x.as_str()).unwrap_or("");
        let tagged = !primaries.is_empty() && !transfer.is_empty() && !matrix.is_empty();
        out.push(RuleResult {
            spec: "Netflix delivery",
            rule: "Color primaries / transfer / matrix tagged",
            verdict: if tagged { Verdict::Pass } else { Verdict::Fail },
            detail: format!(
                "primaries='{primaries}' transfer='{transfer}' matrix='{matrix}'"
            ),
            evidence_view: "streams",
            citation: "Netflix Originals Delivery Spec — Color Tagging",
        });

        // HDR-tagged content should declare BT.2020 + ST 2084 (PQ) or HLG.
        let is_hdr_signaled = transfer == "smpte2084"
            || transfer.contains("hlg")
            || transfer == "arib-std-b67";
        if is_hdr_signaled {
            let bt2020 = primaries == "bt2020" && matrix.starts_with("bt2020");
            out.push(RuleResult {
                spec: "Netflix delivery",
                rule: "HDR signal uses BT.2020 primaries + matrix",
                verdict: if bt2020 { Verdict::Pass } else { Verdict::Fail },
                detail: format!(
                    "HDR transfer '{transfer}' detected; primaries='{primaries}' matrix='{matrix}'"
                ),
                evidence_view: "streams",
                citation: "Netflix HDR Delivery — Container Signaling",
            });
        }
    }

    // Audio sample rate should be 48 kHz for finished masters.
    if let Some(audio) = ff
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|streams| streams.iter().find(|s| s["codec_type"] == "audio"))
    {
        let sr = audio
            .get("sample_rate")
            .and_then(|x| x.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        out.push(RuleResult {
            spec: "Netflix delivery",
            rule: "Audio sample rate = 48 kHz",
            verdict: if sr == 48_000 { Verdict::Pass } else { Verdict::Warn },
            detail: format!("sample_rate = {sr} Hz"),
            evidence_view: "streams",
            citation: "Netflix Audio Delivery Spec",
        });
    }

    out
}
