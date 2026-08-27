use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub selector: String,
    pub extract_mode: String,
    pub threshold: f64,
    pub interval_minutes: i64,
    pub enabled: i64,
    #[serde(skip_serializing)]
    pub baseline: Option<String>,
    pub last_checked: Option<String>,
    pub last_status: String,
    pub last_error: Option<String>,
    pub next_check: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SourceInput {
    pub name: String,
    pub url: String,
    pub selector: Option<String>,
    pub extract_mode: Option<String>,
    pub threshold: Option<f64>,
    pub interval_minutes: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Change {
    pub id: String,
    pub source_id: String,
    pub source_name: String,
    pub source_url: String,
    pub selector: String,
    pub previous_text: String,
    pub current_text: String,
    pub change_ratio: f64,
    pub summary: String,
    pub review_state: String,
    pub useful: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewInput {
    pub review_state: Option<String>,
    pub useful: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub outcome: String,
    pub message: String,
    pub change_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Stats {
    pub sources: i64,
    pub unread: i64,
    pub useful: i64,
    pub rated: i64,
}
