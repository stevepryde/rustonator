use chrono::Utc;

pub fn unix_timestamp() -> i64 {
    Utc::now().timestamp_millis()
}
