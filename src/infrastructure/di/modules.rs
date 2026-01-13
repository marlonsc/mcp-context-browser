use shaku::module;

use crate::adapters::http_client::HttpClientPool;
use crate::admin::service::AdminServiceImpl;
use crate::infrastructure::di::factory::ServiceProvider;
use crate::infrastructure::metrics::system::SystemMetricsCollector;
use crate::server::{McpIndexingOperations, McpPerformanceMetrics};

module! {
    pub McpModule {
        components = [
            // Base components (no dependencies) first
            HttpClientPool,
            ServiceProvider,
            SystemMetricsCollector,
            // Components with dependencies after their dependencies
            McpPerformanceMetrics,
            McpIndexingOperations,
            // AdminServiceImpl depends on all above
            AdminServiceImpl
        ],
        providers = []
    }
}
