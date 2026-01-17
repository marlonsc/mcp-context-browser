//! Metrics Collection Interfaces

pub mod system {
    use async_trait::async_trait;
    use shaku::Interface;
    use mcb_domain::error::Result;

    /// System metrics collector interface
    #[async_trait]
    pub trait SystemMetricsCollectorInterface: Interface + Send + Sync {
        /// Collect current system metrics
        async fn collect(&self) -> Result<SystemMetrics>;

        /// Get CPU usage percentage
        fn cpu_usage(&self) -> f64;

        /// Get memory usage percentage
        fn memory_usage(&self) -> f64;
    }

    /// System metrics data
    #[derive(Debug, Clone, Default)]
    pub struct SystemMetrics {
        pub cpu_percent: f64,
        pub memory_percent: f64,
        pub disk_percent: f64,
    }

    /// Null implementation for testing
    #[derive(shaku::Component)]
    #[shaku(interface = SystemMetricsCollectorInterface)]
    pub struct NullSystemMetricsCollector;

    impl NullSystemMetricsCollector {
        pub fn new() -> Self { Self }
    }

    impl Default for NullSystemMetricsCollector {
        fn default() -> Self { Self::new() }
    }

    #[async_trait]
    impl SystemMetricsCollectorInterface for NullSystemMetricsCollector {
        async fn collect(&self) -> Result<SystemMetrics> {
            Ok(SystemMetrics::default())
        }
        fn cpu_usage(&self) -> f64 { 0.0 }
        fn memory_usage(&self) -> f64 { 0.0 }
    }
}
