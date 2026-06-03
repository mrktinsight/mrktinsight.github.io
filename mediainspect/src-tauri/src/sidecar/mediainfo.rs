use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn which() -> Option<PathBuf> {
    super::find_on_path("mediainfo")
}

pub async fn probe(path: &Path) -> Result<serde_json::Value, AppError> {
    let bin = which().ok_or_else(|| AppError::Sidecar("mediainfo not found on PATH".into()))?;

    let output = tokio::process::Command::new(&bin)
        .args(["--Output=JSON", "--Full"])
        .arg(path)
        .output()
        .await
        .map_err(|e| AppError::Sidecar(format!("spawn mediainfo: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Sidecar(format!(
            "mediainfo exit {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Parse(format!("mediainfo json: {e}")))
}
