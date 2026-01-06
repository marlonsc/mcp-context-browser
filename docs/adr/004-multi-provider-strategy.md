# ADR 004: Multi-Provider Strategy

## Status

Accepted

## Context

The MCP Context Browser depends on external AI and storage services that have varying reliability, cost structures, and performance characteristics. Single-provider architectures create vendor lock-in, single points of failure, and cost optimization challenges.

External dependencies and risks:

-   **AI Providers**: OpenAI (expensive, reliable), Ollama (free, local), Anthropic (premium)
-   **Vector Databases**: Milvus (scalable, complex), Pinecone (managed, expensive), Qdrant (simple, limited scale)
-   **Service Outages**: Any provider can experience downtime
-   **API Limits**: Rate limits, quotas, and cost controls needed
-   **Performance Variation**: Different providers have different latency characteristics
-   **Cost Optimization**: Need to balance cost vs. quality vs. speed

The system needs to be resilient, cost-effective, and performant while avoiding vendor lock-in.

## Decision

Implement a multi-provider strategy with automatic failover, load balancing, and provider selection based on context. The system will support multiple providers for each service type with intelligent routing and fallback mechanisms.

Key architectural elements:

-   **Provider Health Monitoring**: Continuous monitoring of provider availability and performance
-   **Intelligent Routing**: Context-aware provider selection (cost, speed, quality)
-   **Automatic Failover**: Seamless fallback to alternative providers
-   **Load Balancing**: Distribute load across multiple provider instances
-   **Cost Tracking**: Monitor and optimize provider usage costs
-   **Configuration Flexibility**: Runtime provider switching and reconfiguration

## Consequences

Multi-provider strategy provides excellent resilience and flexibility but adds significant operational complexity.

### Positive Consequences

-   **High Availability**: No single points of failure for external services
-   **Cost Optimization**: Choose providers based on cost/performance trade-offs
-   **Performance Optimization**: Route to fastest available provider
-   **Future-Proofing**: Easy to add new providers as they emerge
-   **Resilience**: Automatic failover during provider outages
-   **Quality Control**: Select providers based on use case requirements

### Negative Consequences

-   **Operational Complexity**: Managing multiple provider configurations
-   **Development Overhead**: Additional abstraction layers and error handling
-   **Testing Complexity**: Need to test with multiple provider combinations
-   **Cost Management**: Additional complexity in tracking and optimizing costs
-   **Configuration Complexity**: More configuration options and potential misconfigurations
-   **Performance Overhead**: Routing and monitoring add latency

## Alternatives Considered

### Alternative 1: Single Provider Architecture

-   **Description**: Use one primary provider for each service type
-   **Pros**: Simpler implementation, easier configuration, predictable costs
-   **Cons**: Vendor lock-in, single point of failure, limited flexibility
-   **Rejection Reason**: Creates unacceptable availability and cost risks

### Alternative 2: Provider Abstraction Only

-   **Description**: Abstract providers but still use single provider at runtime
-   **Pros**: Ready for multi-provider, simpler initial implementation
-   **Cons**: Doesn't solve availability issues, still vendor-dependent
-   **Rejection Reason**: Doesn't provide the resilience and flexibility needed

### Alternative 3: Provider Mesh with Manual Failover

-   **Description**: Support multiple providers but require manual intervention for failover
-   **Pros**: Simpler than automatic failover, still provides flexibility
-   **Cons**: Slow recovery from outages, requires on-call intervention
-   **Rejection Reason**: Doesn't meet availability requirements for production system

## Implementation Notes

### Provider Selection Strategy

```rust
#[derive(Clone)]
pub enum ProviderSelectionStrategy {
    /// Always use the fastest available provider
    Fastest,
    /// Use the cheapest provider that meets quality thresholds
    Cheapest,
    /// Distribute load across all healthy providers
    LoadBalanced,
    /// Use specific provider for specific use cases
    Contextual,
    /// Custom selection logic
    Custom(Box<dyn ProviderSelector>),
}

pub struct ProviderRouter<P: Provider> {
    providers: HashMap<String, Arc<P>>,
    health_monitor: Arc<HealthMonitor>,
    selection_strategy: ProviderSelectionStrategy,
    metrics_collector: Arc<MetricsCollector>,
}

impl<P: Provider> ProviderRouter<P> {
    pub async fn select_provider(&self, context: &ProviderContext) -> Result<Arc<P>> {
        let healthy_providers = self.get_healthy_providers().await?;

        match &self.selection_strategy {
            ProviderSelectionStrategy::Fastest => {
                self.select_fastest_provider(&healthy_providers, context).await
            }
            ProviderSelectionStrategy::Cheapest => {
                self.select_cheapest_provider(&healthy_providers, context).await
            }
            ProviderSelectionStrategy::LoadBalanced => {
                self.select_load_balanced_provider(&healthy_providers).await
            }
            ProviderSelectionStrategy::Contextual => {
                self.select_contextual_provider(&healthy_providers, context).await
            }
            ProviderSelectionStrategy::Custom(selector) => {
                selector.select_provider(&healthy_providers, context).await
            }
        }
    }
}
```

### Health Monitoring and Failover

```rust
#[derive(Clone)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error_rate: f64,
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
}

pub struct HealthMonitor {
    health_checks: HashMap<String, ProviderHealth>,
    check_interval: Duration,
    failure_threshold: u32,
}

impl HealthMonitor {
    pub async fn monitor_provider(&self, provider_id: &str) -> Result<()> {
        let health = self.perform_health_check(provider_id).await?;

        if health.status == HealthStatus::Unhealthy {
            self.handle_provider_failure(provider_id, &health).await?;
        } else {
            self.update_provider_health(provider_id, health).await?;
        }

        Ok(())
    }

    async fn perform_health_check(&self, provider_id: &str) -> Result<ProviderHealth> {
        let start_time = Instant::now();

        // Perform lightweight health check (e.g., API ping)
        let check_result = self.execute_health_check(provider_id).await;
        let latency = start_time.elapsed().as_millis() as u64;

        let (status, error_rate) = match check_result {
            Ok(_) => (HealthStatus::Healthy, 0.0),
            Err(_) => (HealthStatus::Unhealthy, 1.0),
        };

        Ok(ProviderHealth {
            provider_id: provider_id.to_string(),
            status,
            latency_ms: Some(latency),
            error_rate,
            last_check: Utc::now(),
            consecutive_failures: if status == HealthStatus::Unhealthy { 1 } else { 0 },
        })
    }
}
```

### Cost Tracking and Optimization

```rust
#[derive(Clone)]
pub struct ProviderCost {
    pub provider_id: String,
    pub operation_type: String,
    pub cost_per_unit: f64,
    pub unit_type: String,  // "token", "request", "GB", etc.
    pub free_tier_limit: Option<u64>,
}

pub struct CostTracker {
    costs: HashMap<String, ProviderCost>,
    usage_metrics: HashMap<String, UsageMetrics>,
    budget_limits: HashMap<String, f64>,
}

impl CostTracker {
    pub async fn track_operation_cost(
        &self,
        provider_id: &str,
        operation: &str,
        units: u64,
    ) -> Result<f64> {
        let cost_info = self.costs.get(provider_id)
            .ok_or_else(|| Error::not_found(format!("Cost info for provider: {}", provider_id)))?;

        let total_cost = if let Some(free_limit) = cost_info.free_tier_limit {
            if units <= free_limit {
                0.0
            } else {
                (units - free_limit) as f64 * cost_info.cost_per_unit
            }
        } else {
            units as f64 * cost_info.cost_per_unit
        };

        // Check budget limits
        if let Some(budget_limit) = self.budget_limits.get(provider_id) {
            let current_usage = self.get_current_monthly_cost(provider_id).await?;
            if current_usage + total_cost > *budget_limit {
                return Err(Error::budget_exceeded(format!(
                    "Budget limit exceeded for provider: {}", provider_id
                )));
            }
        }

        // Record usage
        self.record_usage(provider_id, operation, units, total_cost).await?;

        Ok(total_cost)
    }
}
```

### Configuration Management

```toml
# config/providers.toml
[providers]

[providers.embedding]
default_provider = "openai"
fallback_providers = ["ollama", "mock"]

[providers.embedding.openai]
api_key = "${OPENAI_API_KEY}"
model = "text-embedding-3-small"
cost_per_1k_tokens = 0.0001
free_tier_limit = 1000000

[providers.embedding.ollama]
base_url = "http://localhost:11434"
model = "nomic-embed-text"
cost_per_1k_tokens = 0.0

[providers.vector_store]
default_provider = "milvus"
fallback_providers = ["memory"]

[providers.vector_store.milvus]
uri = "localhost:19530"
token = "${MILVUS_TOKEN}"
cost_per_gb = 0.0

[routing]
strategy = "contextual"

[routing.contextual.embedding]
fast_response = "ollama"    # For quick responses, use local model
high_quality = "openai"     # For quality-critical tasks, use OpenAI
cost_optimized = "ollama"   # For bulk processing, use free tier

[routing.contextual.vector_store]
development = "memory"      # Use in-memory for development
production = "milvus"       # Use Milvus for production
```

### Circuit Breaker Pattern

```rust
pub struct ProviderCircuitBreaker {
    provider_id: String,
    state: CircuitBreakerState,
    config: CircuitBreakerConfig,
    metrics: Arc<MetricsCollector>,
}

#[derive(Clone)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

impl ProviderCircuitBreaker {
    pub async fn call<T, F>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match &self.state {
            CircuitBreakerState::Open { opened_at } => {
                if opened_at.elapsed() > self.config.timeout_duration {
                    // Try again in half-open state
                    *self.state.write().await = CircuitBreakerState::HalfOpen;
                } else {
                    return Err(Error::circuit_breaker_open(&self.provider_id));
                }
            }
            _ => {}
        }

        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    async fn on_success(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        self.metrics.record_circuit_breaker_success(&self.provider_id);
    }

    async fn on_failure(&self) {
        let mut failures = self.failure_count.write().await;
        *failures += 1;

        if *failures >= self.config.failure_threshold {
            *self.state.write().await = CircuitBreakerState::Open {
                opened_at: Instant::now(),
            };
        }

        self.metrics.record_circuit_breaker_failure(&self.provider_id);
    }
}
```

## References

-   [Circuit Breaker Pattern](https://microservices.io/patterns/reliability/circuit-breaker.html)
-   [Provider Selection Strategies](https://aws.amazon.com/blogs/architecture/)
-   [Multi-Cloud Architecture Patterns](https://aws.amazon.com/blogs/architecture/category/multi-cloud/)
