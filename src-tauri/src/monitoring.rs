use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::Serialize;

/// Real-time monitoring data returned by the stats endpoint.
#[derive(Serialize)]
pub struct MonitoringData {
    pub total_requests: u64,
    pub current_connections: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub uptime: String,
}

/// Global counters for request monitoring.
static TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static CURRENT_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
static SUCCESS_COUNT: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);

/// Server start time, initialised on first access.
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Must be called once when the server starts listening.
pub fn init() {
    START_TIME.get_or_init(Instant::now);
}

/// Increment total request counter.
pub fn increment_requests() {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
}

/// Record a successful response.
pub fn record_success() {
    SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Record an error response.
pub fn record_error() {
    ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Increment active connection count.
pub fn connection_opened() {
    CURRENT_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
}

/// Decrement active connection count.
#[allow(dead_code)]
pub fn connection_closed() {
    CURRENT_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
}

/// Format a `Duration` into a human-readable string like "2 days, 5:30:15".
fn format_uptime(elapsed: std::time::Duration) -> String {
    let total_secs = elapsed.as_secs();
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{} days, {}:{:02}:{:02}", days, hours, minutes, seconds)
    } else {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// Snapshot the current monitoring data.
pub fn get_monitoring_data() -> MonitoringData {
    let total = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let success = SUCCESS_COUNT.load(Ordering::Relaxed);
    let errors = ERROR_COUNT.load(Ordering::Relaxed);

    let success_rate = if total > 0 {
        (success as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    let uptime = START_TIME
        .get()
        .map(|t| format_uptime(t.elapsed()))
        .unwrap_or_else(|| "0:00:00".to_string());

    MonitoringData {
        total_requests: total,
        current_connections: CURRENT_CONNECTIONS.load(Ordering::Relaxed),
        success_count: success,
        error_count: errors,
        success_rate,
        uptime,
    }
}
