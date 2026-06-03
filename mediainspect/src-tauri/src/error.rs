use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(String),

    #[error("sidecar error: {0}")]
    Sidecar(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
