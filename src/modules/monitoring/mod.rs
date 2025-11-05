//! 监控模块
//! 实现服务器监控和统计功能

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ServerStats {
    // 请求统计
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    
    // 流量统计
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
    
    // 连接统计
    pub active_connections: AtomicUsize,
    pub total_connections: AtomicU64,
    
    // 时间统计
    pub start_time: u64,
}

impl ServerStats {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_in(&self, bytes: u64) {
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_bytes_out(&self, bytes: u64) {
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) -> usize {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn decrement_connections(&self) -> usize {
        let current = self.active_connections.fetch_sub(1, Ordering::Relaxed);
        if current > 0 {
            current - 1
        } else {
            0
        }
    }

    pub fn get_stats(&self) -> StatsSnapshot {
        StatsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            bytes_in: self.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.bytes_out.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            total_connections: self.total_connections.load(Ordering::Relaxed),
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0) - self.start_time,
        }
    }
}

pub struct StatsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub active_connections: usize,
    pub total_connections: u64,
    pub uptime: u64,
}

impl StatsSnapshot {
    pub fn requests_per_second(&self) -> f64 {
        if self.uptime > 0 {
            self.total_requests as f64 / self.uptime as f64
        } else {
            0.0
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_errors as f64 / self.total_requests as f64 * 100.0
        } else {
            0.0
        }
    }
}