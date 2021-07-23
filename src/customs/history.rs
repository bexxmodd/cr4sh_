use chrono::{DateTime, Utc};

pub struct HistoryEntry {
    line: String,
    timestamp: DateTime<Utc>,
}