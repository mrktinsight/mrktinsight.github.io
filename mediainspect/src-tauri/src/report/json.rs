use crate::error::AppError;
use crate::InspectReport;

pub fn to_pretty(report: &InspectReport) -> Result<String, AppError> {
    serde_json::to_string_pretty(report).map_err(|e| AppError::Parse(format!("json: {e}")))
}
