pub mod ffmpeg;
pub mod mediainfo;

use std::path::PathBuf;

/// Find a binary on PATH. In a packaged build we'll also look next to the
/// app executable for bundled binaries before falling back to PATH.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.to_string()
            });
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    which_on_path(name)
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        });
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
