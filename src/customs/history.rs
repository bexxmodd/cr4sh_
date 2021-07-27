#![allow(dead_code)]
use chrono::{DateTime, Utc};

struct HistEntry {
    line: String,
    timestamp: DateTime<Utc>,
    
}