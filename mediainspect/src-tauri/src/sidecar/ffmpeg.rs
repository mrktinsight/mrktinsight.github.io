use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn which() -> Option<PathBuf> {
    super::find_on_path("ffprobe")
}

/// Run `ffprobe -show_format -show_streams -show_chapters -show_programs
/// -of json`. We do not interpret here — that's the UI's job — we just
/// parse the JSON so the frontend can rely on `serde_json::Value` shape.
pub async fn probe(path: &Path) -> Result<serde_json::Value, AppError> {
    let bin = which().ok_or_else(|| AppError::Sidecar("ffprobe not found on PATH".into()))?;

    let output = tokio::process::Command::new(&bin)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            "-show_chapters",
            "-show_programs",
            "-show_error",
        ])
        .arg(path)
        .output()
        .await
        .map_err(|e| AppError::Sidecar(format!("spawn ffprobe: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Sidecar(format!(
            "ffprobe exit {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Parse(format!("ffprobe json: {e}")))
}

/// Run `ffprobe -show_packets` for a single stream and return the raw
/// packet list. Used by the bitrate timeline. Heavy on large files —
/// callers should bound it.
pub async fn show_packets(path: &Path, stream_index: u32) -> Result<serde_json::Value, AppError> {
    let bin = which().ok_or_else(|| AppError::Sidecar("ffprobe not found".into()))?;

    let output = tokio::process::Command::new(&bin)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-print_format",
            "json",
            "-select_streams",
            &stream_index.to_string(),
            "-show_packets",
            "-show_entries",
            "packet=pts_time,dts_time,duration_time,size,flags",
        ])
        .arg(path)
        .output()
        .await
        .map_err(|e| AppError::Sidecar(format!("spawn ffprobe: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Sidecar(format!(
            "ffprobe packets exit {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Parse(format!("ffprobe packets json: {e}")))
}
