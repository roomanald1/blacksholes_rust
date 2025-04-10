use chrono::{DateTime, Datelike, NaiveDate, Utc};

pub fn time_to_expiry_in_years(date: NaiveDate) -> f64 {
    let diff = date.num_days_from_ce() - Utc::now().date_naive().num_days_from_ce();
    diff as f64 / 365.25
}

pub fn to_date_time(timestamp: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(timestamp / 1000, ((timestamp % 1000) * 1_000_000) as u32).unwrap()
}

pub fn round_to_nearest_1000(num: i32) -> i32 {
    (num + 500) / 1000 * 1000
}
