//! 进程管理模块

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;

pub struct ProcessManager {
    master_pid: u32,
    worker_processes: Vec<WorkerProcess>,
    config: Arc<RwLock<Config>>,
}

pub struct WorkerProcess {
    pid: u32,
    status: WorkerStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    Running,
    Stopped,
    Failed,
}

impl ProcessManager {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            master_pid: std::process::id(),
            worker_processes: Vec::new(),
            config,
        }
    }

    /// 启动工作进程
    pub async fn start_worker_processes(&mut self, count: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("Master process started with PID: {}", self.master_pid);
        
        for i in 0..count {
            self.start_worker_process(i).await?;
        }
        
        Ok(())
    }

    /// 启动单个工作进程
    async fn start_worker_process(&mut self, worker_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // 在实际实现中，我们会在这里创建子进程
        // 为简化起见，我们只是模拟工作进程
        let worker = WorkerProcess {
            pid: self.master_pid + worker_id as u32 + 1,
            status: WorkerStatus::Running,
        };
        
        println!("Started worker process #{} with PID: {}", worker_id, worker.pid);
        self.worker_processes.push(worker);
        
        Ok(())
    }

    /// 监控工作进程状态
    pub async fn monitor_workers(&self) {
        // 在实际实现中，我们会定期检查工作进程状态并重启失败的进程
        for worker in &self.worker_processes {
            match worker.status {
                WorkerStatus::Running => {
                    println!("Worker process {} is running", worker.pid);
                }
                WorkerStatus::Failed => {
                    eprintln!("Worker process {} has failed", worker.pid);
                }
                WorkerStatus::Stopped => {
                    println!("Worker process {} is stopped", worker.pid);
                }
            }
        }
    }

    /// 优雅关闭所有工作进程
    pub async fn shutdown_workers(&mut self) {
        println!("Shutting down all worker processes...");
        for worker in &mut self.worker_processes {
            worker.status = WorkerStatus::Stopped;
            println!("Stopped worker process {}", worker.pid);
        }
    }
}

impl WorkerProcess {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            status: WorkerStatus::Running,
        }
    }
    
    pub fn get_pid(&self) -> u32 {
        self.pid
    }
    
    pub fn get_status(&self) -> &WorkerStatus {
        &self.status
    }
    
    pub fn set_status(&mut self, status: WorkerStatus) {
        self.status = status;
    }
}