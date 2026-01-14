//! System metrics collection using sysinfo crate
//!
//! Provides accurate system monitoring for CPU, memory, disk, network, and process metrics.

use crate::domain::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::Component;
use sysinfo::{Disks, Networks, Pid, ProcessesToUpdate, System};
use tokio::sync::{mpsc, oneshot};

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuMetrics {
    /// CPU usage percentage (0-100)
    pub usage: f32,
    /// Number of CPU cores
    pub cores: usize,
    /// CPU model name
    pub model: String,
    /// CPU frequency in MHz
    pub speed: u64,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetrics {
    /// Total memory in bytes
    pub total: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Free memory in bytes
    pub free: u64,
    /// Memory usage percentage (0-100)
    pub usage_percent: f32,
}

/// Disk usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskMetrics {
    /// Total disk space in bytes
    pub total: u64,
    /// Used disk space in bytes
    pub used: u64,
    /// Available disk space in bytes
    pub available: u64,
    /// Disk usage percentage (0-100)
    pub usage_percent: f32,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// Bytes received since system start
    pub bytes_received: u64,
    /// Bytes sent since system start
    pub bytes_sent: u64,
    /// Packets received since system start
    pub packets_received: u64,
    /// Packets sent since system start
    pub packets_sent: u64,
}

/// Process metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// Memory usage in bytes
    pub memory: u64,
    /// Memory usage percentage
    pub memory_percent: f32,
    /// CPU usage percentage
    pub cpu_percent: f32,
    /// Process uptime in seconds
    pub uptime: u64,
}

/// Messages for the system metrics actor
/// Messages for the system metrics actor
pub enum SystemMetricsMessage {
    /// Request CPU metrics collection
    CollectCpu(oneshot::Sender<Result<CpuMetrics>>),
    /// Request memory metrics collection
    CollectMemory(oneshot::Sender<Result<MemoryMetrics>>),
    /// Request process metrics collection
    CollectProcess(oneshot::Sender<Result<ProcessMetrics>>),
}

/// System metrics collector Interface
#[async_trait]
pub trait SystemMetricsCollectorInterface: shaku::Interface + Send + Sync {
    /// Collect current CPU usage metrics
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics>;
    /// Collect current memory usage metrics
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics>;
    /// Collect current disk usage metrics
    async fn collect_disk_metrics(&self) -> Result<DiskMetrics>;
    /// Collect current network usage metrics
    async fn collect_network_metrics(&self) -> Result<NetworkMetrics>;
    /// Collect current process metrics
    async fn collect_process_metrics(&self) -> Result<ProcessMetrics>;
    /// Collect all system metrics at once
    async fn collect_all_metrics(
        &self,
    ) -> Result<(
        CpuMetrics,
        MemoryMetrics,
        DiskMetrics,
        NetworkMetrics,
        ProcessMetrics,
    )>;
}

/// System metrics collector using Actor pattern to eliminate locks
#[derive(Component)]
#[shaku(interface = SystemMetricsCollectorInterface)]
pub struct SystemMetricsCollector {
    /// Channel sender for sending metrics collection requests to the actor
    #[shaku(default = SystemMetricsCollector::new().sender)]
    sender: mpsc::Sender<SystemMetricsMessage>,
}

#[async_trait]
impl SystemMetricsCollectorInterface for SystemMetricsCollector {
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(SystemMetricsMessage::CollectCpu(tx)).await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(SystemMetricsMessage::CollectMemory(tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn collect_disk_metrics(&self) -> Result<DiskMetrics> {
        // Disks::new_with_refreshed_list() is sync but doesn't require System
        let disks = Disks::new_with_refreshed_list();
        if disks.is_empty() {
            return Err(Error::internal("No disk information available"));
        }

        let mut total_space = 0u64;
        let mut available_space = 0u64;

        for disk in &disks {
            total_space += disk.total_space();
            available_space += disk.available_space();
        }

        let used_space = total_space.saturating_sub(available_space);
        let usage_percent = if total_space > 0 {
            (used_space as f32 / total_space as f32) * 100.0
        } else {
            0.0
        };

        Ok(DiskMetrics {
            total: total_space,
            used: used_space,
            available: available_space,
            usage_percent,
        })
    }

    async fn collect_network_metrics(&self) -> Result<NetworkMetrics> {
        let networks = Networks::new_with_refreshed_list();
        let mut bytes_received = 0u64;
        let mut bytes_sent = 0u64;
        let mut packets_received = 0u64;
        let mut packets_sent = 0u64;

        for (_interface_name, network) in &networks {
            bytes_received += network.received();
            bytes_sent += network.transmitted();
            // sysinfo doesn't provide packet counts directly in simple way, so we estimate
            packets_received += network.received() / 1500; // Rough estimate
            packets_sent += network.transmitted() / 1500;
        }

        Ok(NetworkMetrics {
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
        })
    }

    async fn collect_process_metrics(&self) -> Result<ProcessMetrics> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(SystemMetricsMessage::CollectProcess(tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn collect_all_metrics(
        &self,
    ) -> Result<(
        CpuMetrics,
        MemoryMetrics,
        DiskMetrics,
        NetworkMetrics,
        ProcessMetrics,
    )> {
        let cpu = self.collect_cpu_metrics().await?;
        let memory = self.collect_memory_metrics().await?;
        let disk = self.collect_disk_metrics().await?;
        let network = self.collect_network_metrics().await?;
        let process = self.collect_process_metrics().await?;

        Ok((cpu, memory, disk, network, process))
    }
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut actor = SystemMetricsActor::new(rx);
        tokio::spawn(async move {
            actor.run().await;
        });

        Self { sender: tx }
    }
}

struct SystemMetricsActor {
    receiver: mpsc::Receiver<SystemMetricsMessage>,
    system: System,
}

impl SystemMetricsActor {
    fn new(receiver: mpsc::Receiver<SystemMetricsMessage>) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { receiver, system }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                SystemMetricsMessage::CollectCpu(tx) => {
                    self.system.refresh_cpu_all();
                    let cpus = self.system.cpus();
                    if cpus.is_empty() {
                        let _ = tx.send(Err(Error::internal("No CPU information available")));
                        continue;
                    }
                    let usage =
                        cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
                    let cores = cpus.len();
                    let model = cpus[0].brand().to_string();
                    let speed = cpus[0].frequency();
                    let _ = tx.send(Ok(CpuMetrics {
                        usage,
                        cores,
                        model,
                        speed,
                    }));
                }
                SystemMetricsMessage::CollectMemory(tx) => {
                    self.system.refresh_memory();
                    let total = self.system.total_memory();
                    let used = self.system.used_memory();
                    let free = self.system.free_memory();
                    let usage_percent = if total > 0 {
                        (used as f32 / total as f32) * 100.0
                    } else {
                        0.0
                    };
                    let _ = tx.send(Ok(MemoryMetrics {
                        total,
                        used,
                        free,
                        usage_percent,
                    }));
                }
                SystemMetricsMessage::CollectProcess(tx) => {
                    let pid = std::process::id();
                    self.system.refresh_processes(
                        ProcessesToUpdate::Some(&[Pid::from(pid as usize)]),
                        true,
                    );
                    if let Some(process) = self.system.process(Pid::from(pid as usize)) {
                        let memory = process.memory();
                        let cpu_percent = process.cpu_usage();
                        let uptime = process.run_time();
                        let total_memory = self.system.total_memory();
                        let memory_percent = if total_memory > 0 {
                            (memory as f32 / total_memory as f32) * 100.0
                        } else {
                            0.0
                        };
                        let _ = tx.send(Ok(ProcessMetrics {
                            pid,
                            memory,
                            memory_percent,
                            cpu_percent,
                            uptime,
                        }));
                    } else {
                        let _ = tx.send(Err(Error::internal(format!(
                            "Process with PID {} not found",
                            pid
                        ))));
                    }
                }
            }
        }
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
