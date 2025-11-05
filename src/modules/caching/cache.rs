//! 缓存实现

use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use hyper::{Response, StatusCode, header};
use http_body_util::Full;
use std::sync::RwLock;

pub struct CacheEntry {
    pub response: Response<Full<bytes::Bytes>>,
    pub expires_at: SystemTime,
    pub etag: Option<String>,
}

pub struct HttpCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl HttpCache {
    pub fn new(default_ttl_seconds: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(default_ttl_seconds),
        }
    }

    pub fn get(&self, key: &str) -> Option<Response<Full<bytes::Bytes>>> {
        let now = SystemTime::now();
        let entries = self.entries.read().unwrap();
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > now {
                return Some(entry.response.clone());
            }
        }
        None
    }

    pub fn put(&self, key: String, response: Response<Full<bytes::Bytes>>) -> Result<(), Box<dyn std::error::Error>> {
        let expires_at = SystemTime::now() + self.default_ttl;
        
        // 生成 ETag
        let etag = {
            let body = response.body();
            let body_bytes = body.collect().await?.to_bytes();
            let hash = format!("{:x}", md5::compute(&body_bytes));
            Some(hash)
        };
        
        let entry = CacheEntry {
            response,
            expires_at,
            etag,
        };
        
        let mut entries = self.entries.write().unwrap();
        entries.insert(key, entry);
        Ok(())
    }

    pub fn remove_expired(&self) {
        let now = SystemTime::now();
        let mut entries = self.entries.write().unwrap();
        entries.retain(|_, entry| entry.expires_at > now);
    }

    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }
    
    pub fn get_etag(&self, key: &str) -> Option<String> {
        let entries = self.entries.read().unwrap();
        entries.get(key).and_then(|entry| entry.etag.clone())
    }
}