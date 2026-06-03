pub mod analysis;
pub mod compliance;
pub mod error;
pub mod probe;
pub mod report;
pub mod sidecar;

use std::path::PathBuf;

use serde::Serialize;

use crate::error::AppError;

/// Top-level report we return to the UI when a file is opened.
/// Each section is independent: the UI shows what's present and skips what
/// isn't, so a missing `mediainfo` binary doesn't break ffprobe results.
#[derive(Debug, Serialize, Default)]
pub struct InspectReport {
    pub path: String,
    pub size_bytes: u64,
    pub ffprobe: Option<serde_json::Value>,
    pub mediainfo: Option<serde_json::Value>,
    pub atoms: Option<probe::isobmff::AtomNode>,
    pub bitrate_timeline: Option<analysis::bitrate::BitrateTimeline>,
    pub loudness: Option<analysis::loudness::LoudnessReport>,
    pub compliance: Vec<compliance::RuleResult>,
    pub warnings: Vec<String>,
}

#[tauri::command]
async fn inspect(path: String) -> Result<InspectReport, AppError> {
    let path_buf = PathBuf::from(&path);
    let metadata = std::fs::metadata(&path_buf)
        .map_err(|e| AppError::Io(format!("stat {}: {}", path, e)))?;

    let mut report = InspectReport {
        path,
        size_bytes: metadata.len(),
        ..Default::default()
    };

    match sidecar::ffmpeg::probe(&path_buf).await {
        Ok(value) => report.ffprobe = Some(value),
        Err(e) => report.warnings.push(format!("ffprobe unavailable: {e}")),
    }

    match sidecar::mediainfo::probe(&path_buf).await {
        Ok(value) => report.mediainfo = Some(value),
        Err(e) => report.warnings.push(format!("mediainfo unavailable: {e}")),
    }

    match probe::isobmff::walk(&path_buf) {
        Ok(Some(tree)) => report.atoms = Some(tree),
        Ok(None) => {} // not an ISOBMFF file; that's fine
        Err(e) => report.warnings.push(format!("atom walker: {e}")),
    }

    if let Some(ff) = &report.ffprobe {
        match analysis::bitrate::from_ffprobe(ff) {
            Ok(timeline) => report.bitrate_timeline = Some(timeline),
            Err(e) => report.warnings.push(format!("bitrate timeline: {e}")),
        }
    }

    match analysis::loudness::measure(&path_buf) {
        Ok(Some(l)) => report.loudness = Some(l),
        Ok(None) => {} // not an audio source we decoded
        Err(e) => report.warnings.push(format!("loudness: {e}")),
    }

    report.compliance = compliance::run_all(&report);

    Ok(report)
}

#[tauri::command]
fn tool_status() -> serde_json::Value {
    serde_json::json!({
        "ffprobe": sidecar::ffmpeg::which().is_some(),
        "mediainfo": sidecar::mediainfo::which().is_some(),
        "ffprobe_path": sidecar::ffmpeg::which(),
        "mediainfo_path": sidecar::mediainfo::which(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mediainspect_lib=debug".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![inspect, tool_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
