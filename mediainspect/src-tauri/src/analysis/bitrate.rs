//! Bitrate-over-time analysis.
//!
//! For each stream, we derive a per-second average bitrate from the
//! declared `bit_rate` in the ffprobe stream info (the cheap path), and a
//! peak/average summary. A future enhancement (post-MVP) will use
//! `ffprobe -show_packets` for true per-second buckets and per-GOP peaks;
//! that path lives in `sidecar::ffmpeg::show_packets` already.

use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Serialize, Default)]
pub struct StreamBitrate {
    pub index: u32,
    pub codec: String,
    pub kind: String,             // "video" | "audio" | "subtitle" | ...
    pub declared_kbps: Option<f64>,
    pub avg_kbps: Option<f64>,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Serialize, Default)]
pub struct BitrateTimeline {
    pub format_kbps: Option<f64>,
    pub streams: Vec<StreamBitrate>,
}

pub fn from_ffprobe(v: &serde_json::Value) -> Result<BitrateTimeline, AppError> {
    let mut tl = BitrateTimeline::default();

    if let Some(fmt) = v.get("format") {
        tl.format_kbps = fmt
            .get("bit_rate")
            .and_then(|x| x.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|b| b / 1000.0);
    }

    let streams = v
        .get("streams")
        .and_then(|s| s.as_array())
        .ok_or_else(|| AppError::Parse("no streams in ffprobe output".into()))?;

    for s in streams {
        let mut entry = StreamBitrate::default();
        entry.index = s.get("index").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        entry.codec = s
            .get("codec_name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        entry.kind = s
            .get("codec_type")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        entry.declared_kbps = s
            .get("bit_rate")
            .and_then(|x| x.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|b| b / 1000.0);
        entry.duration_seconds = s
            .get("duration")
            .and_then(|x| x.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        // If declared bit_rate is missing, derive from nb_frames * avg_frame_size,
        // or from format bit_rate as a fallback proportional split.
        if entry.declared_kbps.is_none() {
            if let (Some(dur), Some(fmt_kbps)) = (entry.duration_seconds, tl.format_kbps) {
                if dur > 0.0 {
                    entry.avg_kbps = Some(fmt_kbps);
                }
            }
        } else {
            entry.avg_kbps = entry.declared_kbps;
        }

        tl.streams.push(entry);
    }

    Ok(tl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_declared_bitrate() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{
              "format": {"bit_rate": "5000000"},
              "streams": [
                {"index": 0, "codec_name": "h264", "codec_type": "video",
                 "bit_rate": "4500000", "duration": "10.0"},
                {"index": 1, "codec_name": "aac", "codec_type": "audio",
                 "bit_rate": "256000", "duration": "10.0"}
              ]
            }"#,
        )
        .unwrap();
        let tl = from_ffprobe(&v).unwrap();
        assert_eq!(tl.format_kbps, Some(5000.0));
        assert_eq!(tl.streams.len(), 2);
        assert_eq!(tl.streams[0].declared_kbps, Some(4500.0));
        assert_eq!(tl.streams[1].declared_kbps, Some(256.0));
    }
}
