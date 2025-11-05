//! A/B 测试模块

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ABTesting {
    experiments: HashMap<String, Experiment>,
}

pub struct Experiment {
    name: String,
    variants: Vec<Variant>,
    total_requests: AtomicU64,
}

pub struct Variant {
    name: String,
    weight: u32, // 权重，用于流量分配
    backend: String, // 后端地址
}

impl ABTesting {
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
        }
    }

    pub fn add_experiment(&mut self, name: String, variants: Vec<Variant>) {
        let experiment = Experiment {
            name: name.clone(),
            variants,
            total_requests: AtomicU64::new(0),
        };
        
        self.experiments.insert(name, experiment);
    }

    pub fn select_backend(&self, experiment_name: &str, client_identifier: &str) -> Option<String> {
        if let Some(experiment) = self.experiments.get(experiment_name) {
            experiment.total_requests.fetch_add(1, Ordering::Relaxed);
            
            // 基于客户端标识符和总请求数进行哈希计算，选择变体
            let hash = self.calculate_hash(client_identifier, experiment.total_requests.load(Ordering::Relaxed));
            let total_weight: u32 = experiment.variants.iter().map(|v| v.weight).sum();
            
            if total_weight == 0 {
                return None;
            }
            
            let mut cumulative_weight = 0;
            let selected_value = hash % total_weight as u64;
            
            for variant in &experiment.variants {
                cumulative_weight += variant.weight;
                if selected_value < cumulative_weight as u64 {
                    return Some(variant.backend.clone());
                }
            }
        }
        
        None
    }

    fn calculate_hash(&self, client_identifier: &str, request_count: u64) -> u64 {
        // 简单的哈希计算，实际实现中可以使用更复杂的哈希算法
        let mut hash: u64 = 0;
        for byte in client_identifier.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash.wrapping_add(request_count)
    }

    pub fn get_experiment_stats(&self, experiment_name: &str) -> Option<ExperimentStats> {
        if let Some(experiment) = self.experiments.get(experiment_name) {
            let total_requests = experiment.total_requests.load(Ordering::Relaxed);
            
            Some(ExperimentStats {
                name: experiment.name.clone(),
                total_requests,
                variants: experiment.variants.iter().map(|v| VariantInfo {
                    name: v.name.clone(),
                    weight: v.weight,
                    backend: v.backend.clone(),
                }).collect(),
            })
        } else {
            None
        }
    }
}

impl Experiment {
    pub fn new(name: String, variants: Vec<Variant>) -> Self {
        Self {
            name,
            variants,
            total_requests: AtomicU64::new(0),
        }
    }
}

pub struct ExperimentStats {
    pub name: String,
    pub total_requests: u64,
    pub variants: Vec<VariantInfo>,
}

pub struct VariantInfo {
    pub name: String,
    pub weight: u32,
    pub backend: String,
}