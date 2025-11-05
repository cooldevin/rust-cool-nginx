//! IP 访问控制实现

use std::net::IpAddr;
use std::str::FromStr;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use std::sync::RwLock;

pub struct IpFilter {
    allow_list: Vec<IpNetwork>,
    deny_list: Vec<IpNetwork>,
    rate_limiter: RwLock<HashMap<String, RateLimitInfo>>,
    max_requests_per_minute: u32,
}

#[derive(Debug, Clone)]
pub enum IpNetwork {
    V4(Ipv4Network),
    V6(Ipv6Network),
}

#[derive(Debug, Clone)]
pub struct Ipv4Network {
    pub network: std::net::Ipv4Addr,
    pub prefix: u8,
}

#[derive(Debug, Clone)]
pub struct Ipv6Network {
    pub network: std::net::Ipv6Addr,
    pub prefix: u8,
}

// 速率限制信息
#[derive(Debug)]
struct RateLimitInfo {
    request_count: u32,
    last_reset: SystemTime,
}

impl IpFilter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            allow_list: Vec::new(),
            deny_list: Vec::new(),
            rate_limiter: RwLock::new(HashMap::new()),
            max_requests_per_minute,
        }
    }

    pub fn allow(&mut self, network: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ip_network = Self::parse_network(network)?;
        self.allow_list.push(ip_network);
        Ok(())
    }

    pub fn deny(&mut self, network: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ip_network = Self::parse_network(network)?;
        self.deny_list.push(ip_network);
        Ok(())
    }

    pub fn is_allowed(&self, ip: &str) -> bool {
        if let Ok(ip_addr) = IpAddr::from_str(ip) {
            // 先检查拒绝列表
            for network in &self.deny_list {
                if Self::ip_in_network(&ip_addr, network) {
                    return false;
                }
            }

            // 如果有允许列表，必须在允许列表中
            if !self.allow_list.is_empty() {
                for network in &self.allow_list {
                    if Self::ip_in_network(&ip_addr, network) {
                        return true;
                    }
                }
                return false;
            }

            // 默认允许（没有明确的允许列表）
            true
        } else {
            false
        }
    }

    // 检查是否超过速率限制
    pub fn is_rate_limited(&self, ip: &str) -> bool {
        let now = SystemTime::now();
        let mut rate_limiter = self.rate_limiter.write().unwrap();
        
        let info = rate_limiter.entry(ip.to_string()).or_insert(RateLimitInfo {
            request_count: 0,
            last_reset: now,
        });
        
        // 检查是否需要重置计数器（超过1分钟）
        if let Ok(elapsed) = now.duration_since(info.last_reset) {
            if elapsed > Duration::from_secs(60) {
                info.request_count = 0;
                info.last_reset = now;
            }
        }
        
        // 增加请求计数
        info.request_count += 1;
        
        // 检查是否超过限制
        info.request_count > self.max_requests_per_minute
    }

    fn parse_network(network: &str) -> Result<IpNetwork, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = network.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid network format".into());
        }

        let ip = parts[0];
        let prefix: u8 = parts[1].parse()?;

        if let Ok(ipv4) = std::net::Ipv4Addr::from_str(ip) {
            if prefix > 32 {
                return Err("Invalid IPv4 prefix".into());
            }
            Ok(IpNetwork::V4(Ipv4Network {
                network: ipv4,
                prefix,
            }))
        } else if let Ok(ipv6) = std::net::Ipv6Addr::from_str(ip) {
            if prefix > 128 {
                return Err("Invalid IPv6 prefix".into());
            }
            Ok(IpNetwork::V6(Ipv6Network {
                network: ipv6,
                prefix,
            }))
        } else {
            Err("Invalid IP address".into())
        }
    }

    fn ip_in_network(ip: &IpAddr, network: &IpNetwork) -> bool {
        match (ip, network) {
            (IpAddr::V4(ipv4), IpNetwork::V4(net)) => {
                if net.prefix == 0 {
                    return true;
                }
                let mask = !0u32 << (32 - net.prefix);
                let ip_bits = u32::from(*ipv4);
                let network_bits = u32::from(net.network);
                (ip_bits & mask) == (network_bits & mask)
            }
            (IpAddr::V6(ipv6), IpNetwork::V6(net)) => {
                if net.prefix == 0 {
                    return true;
                }
                // 简化处理，实际实现应该更复杂
                let ip_segments = ipv6.segments();
                let network_segments = net.network.segments();
                // 这里仅作示意，实际需要正确实现 IPv6 网络匹配
                ip_segments[0] == network_segments[0]
            }
            _ => false,
        }
    }
}