use chrono::prelude;

pub struct HistoryEntry {
    line: String,
    timestamp: DateTime<Utc>,
}