//! 负载均衡器实现

use std::collections::HashMap;
use std::sync::Arc;
use crate::modules::load_balancing::algorithms::{LoadBalancingAlgorithm, RoundRobin};

pub struct LoadBalancer {
    algorithm: Box<dyn LoadBalancingAlgorithm>,
    backends: Vec<String>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            algorithm: Box::new(RoundRobin::new()),
            backends: Vec::new(),
        }
    }

    pub fn with_algorithm(mut self, algorithm: Box<dyn LoadBalancingAlgorithm>) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn add_backend(&mut self, backend: String) {
        self.backends.push(backend);
    }

    pub fn select_backend(&self) -> Option<String> {
        self.algorithm.select_backend(&self.backends)
    }

    pub fn remove_backend(&mut self, backend: &str) {
        self.backends.retain(|b| b != backend);
    }

    pub fn get_backends(&self) -> &[String] {
        &self.backends
    }
}