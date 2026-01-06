//! System metrics collection using sysinfo crate
//!
//! Provides accurate system monitoring for CPU, memory, disk, network, and process metrics.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, DiskExt, NetworkExt, ProcessExt, System, SystemExt};

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// System metrics collector
pub struct SystemMetricsCollector {
    system: System,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self { system }
    }

    /// Collect CPU metrics
    pub fn collect_cpu_metrics(&mut self) -> Result<CpuMetrics> {
        self.system.refresh_cpu();

        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return Err(Error::internal("No CPU information available"));
        }

        let usage = cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        let cores = cpus.len();

        // Get CPU model from first core
        let model = cpus[0].brand().to_string();

        // Estimate CPU speed (sysinfo doesn't provide this directly)
        let speed = cpus[0].frequency();

        Ok(CpuMetrics {
            usage,
            cores,
            model,
            speed,
        })
    }

    /// Collect memory metrics
    pub fn collect_memory_metrics(&mut self) -> Result<MemoryMetrics> {
        self.system.refresh_memory();

        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let free = self.system.free_memory();
        let usage_percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

        Ok(MemoryMetrics {
            total,
            used,
            free,
            usage_percent,
        })
    }

    /// Collect disk metrics (aggregate all disks)
    pub fn collect_disk_metrics(&mut self) -> Result<DiskMetrics> {
        self.system.refresh_disks();

        let disks = self.system.disks();
        if disks.is_empty() {
            return Err(Error::internal("No disk information available"));
        }

        let mut total_space = 0u64;
        let mut available_space = 0u64;

        for disk in disks {
            total_space += disk.total_space();
            available_space += disk.available_space();
        }

        let used_space = total_space.saturating_sub(available_space);
        let usage_percent = if total_space > 0 { (used_space as f32 / total_space as f32) * 100.0 } else { 0.0 };

        Ok(DiskMetrics {
            total: total_space,
            used: used_space,
            available: available_space,
            usage_percent,
        })
    }

    /// Collect network metrics (aggregate all interfaces)
    pub fn collect_network_metrics(&mut self) -> Result<NetworkMetrics> {
        self.system.refresh_networks();

        let networks = self.system.networks();
        let mut bytes_received = 0u64;
        let mut bytes_sent = 0u64;
        let mut packets_received = 0u64;
        let mut packets_sent = 0u64;

        for (_interface_name, network) in networks {
            bytes_received += network.received();
            bytes_sent += network.transmitted();
            // sysinfo doesn't provide packet counts, so we estimate
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

    /// Collect process metrics for current process
    pub fn collect_process_metrics(&mut self) -> Result<ProcessMetrics> {
        self.system.refresh_processes();

        let pid = std::process::id();

        if let Some(process) = self.system.process(pid) {
            let memory = process.memory();
            let cpu_percent = process.cpu_usage();
            let uptime = process.run_time();

            // Calculate memory percentage
            let total_memory = self.system.total_memory();
            let memory_percent = if total_memory > 0 { (memory as f32 / total_memory as f32) * 100.0 } else { 0.0 };

            Ok(ProcessMetrics {
                pid,
                memory,
                memory_percent,
                cpu_percent,
                uptime,
            })
        } else {
            Err(Error::internal(format!("Process with PID {} not found", pid)))
        }
    }

    /// Collect all system metrics at once
    pub fn collect_all_metrics(&mut self) -> Result<(
        CpuMetrics,
        MemoryMetrics,
        DiskMetrics,
        NetworkMetrics,
        ProcessMetrics,
    )> {
        let cpu = self.collect_cpu_metrics()?;
        let memory = self.collect_memory_metrics()?;
        let disk = self.collect_disk_metrics()?;
        let network = self.collect_network_metrics()?;
        let process = self.collect_process_metrics()?;

        Ok((cpu, memory, disk, network, process))
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}