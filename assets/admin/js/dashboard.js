// Dashboard management for admin interface

class DashboardManager {
    constructor() {
        this.charts = {};
        this.metricsInterval = null;
        this.init();
    }

    init() {
        this.startMetricsUpdates();
    }

    async loadDashboard() {
        try {
            // Load all dashboard data in parallel
            const [health, status, providers, metrics] = await Promise.all([
                API.getHealth(),
                API.getStatus(),
                API.getProviders(),
                API.getMetrics()
            ]);

            // Update UI components
            this.updateHealthStatus(health.data);
            this.updateSystemStatus(status.data);
            this.updateProvidersStatus(providers.data);
            this.updateMetrics(metrics.data);

            // Update charts
            this.updateCharts(metrics.data);

        } catch (error) {
            console.error('Failed to load dashboard:', error);
            this.showError('Failed to load dashboard data');
        }
    }

    updateHealthStatus(health) {
        const uptimeEl = document.getElementById('uptime');
        const pidEl = document.getElementById('pid');
        const statusEl = document.querySelector('.status-indicator');

        if (uptimeEl) uptimeEl.textContent = this.formatUptime(health.uptime);
        if (pidEl) pidEl.textContent = health.pid;

        if (statusEl) {
            statusEl.textContent = health.status;
            statusEl.className = `status-indicator status-${health.status.toLowerCase()}`;
        }
    }

    updateSystemStatus(status) {
        const activeIndexesEl = document.getElementById('active-indexes');
        const totalDocumentsEl = document.getElementById('total-documents');

        if (activeIndexesEl) {
            activeIndexesEl.textContent = status.indexes?.active || 0;
        }

        if (totalDocumentsEl) {
            totalDocumentsEl.textContent = status.indexes?.total_docs || 0;
        }
    }

    updateProvidersStatus(providers) {
        const activeProvidersEl = document.getElementById('active-providers');
        const totalProvidersEl = document.getElementById('total-providers');

        const activeCount = providers.filter(p => p.status === 'active').length;

        if (activeProvidersEl) activeProvidersEl.textContent = activeCount;
        if (totalProvidersEl) totalProvidersEl.textContent = providers.length;
    }

    updateMetrics(metrics) {
        const cpuEl = document.getElementById('cpu-usage');
        const memoryEl = document.getElementById('memory-usage');

        if (cpuEl && metrics.cpu) {
            cpuEl.textContent = `${metrics.cpu.usage.toFixed(1)}%`;
        }

        if (memoryEl && metrics.memory) {
            memoryEl.textContent = `${metrics.memory.usage_percent.toFixed(1)}%`;
        }
    }

    updateCharts(metrics) {
        if (metrics.cpu && metrics.memory) {
            this.updateMetricsChart(metrics);
        }

        if (metrics.query_performance) {
            this.updateQueryChart(metrics.query_performance);
        }
    }

    updateMetricsChart(metrics) {
        // This would update an existing Chart.js instance
        // For now, we'll recreate the chart with new data
        const ctx = document.getElementById('metrics-chart');
        if (!ctx || typeof Chart === 'undefined') return;

        // Destroy existing chart if it exists
        if (this.charts.metrics) {
            this.charts.metrics.destroy();
        }

        this.charts.metrics = new Chart(ctx, {
            type: 'line',
            data: {
                labels: this.generateTimeLabels(6),
                datasets: [{
                    label: 'CPU Usage (%)',
                    data: [12, 19, 15, 25, 22, metrics.cpu.usage],
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Memory Usage (%)',
                    data: [45, 52, 48, 61, 55, metrics.memory.usage_percent],
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });
    }

    updateQueryChart(queryPerformance) {
        const ctx = document.getElementById('query-chart');
        if (!ctx || typeof Chart === 'undefined') return;

        // Destroy existing chart if it exists
        if (this.charts.query) {
            this.charts.query.destroy();
        }

        this.charts.query = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: ['Search', 'Index', 'Status', 'Clear'],
                datasets: [{
                    label: 'Requests per minute',
                    data: [
                        queryPerformance.total_queries || 0,
                        queryPerformance.index_operations || 0,
                        queryPerformance.status_checks || 0,
                        queryPerformance.clear_operations || 0
                    ],
                    backgroundColor: [
                        'rgba(37, 99, 235, 0.8)',
                        'rgba(16, 185, 129, 0.8)',
                        'rgba(245, 158, 11, 0.8)',
                        'rgba(239, 68, 68, 0.8)'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    generateTimeLabels(count) {
        const labels = [];
        const now = new Date();

        for (let i = count - 1; i >= 0; i--) {
            const time = new Date(now.getTime() - i * 60000); // 1 minute intervals
            labels.push(time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
        }

        return labels;
    }

    startMetricsUpdates() {
        // Update metrics every 30 seconds
        this.metricsInterval = setInterval(() => {
            this.loadDashboard();
        }, 30000);
    }

    stopMetricsUpdates() {
        if (this.metricsInterval) {
            clearInterval(this.metricsInterval);
            this.metricsInterval = null;
        }
    }

    formatUptime(seconds) {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (days > 0) {
            return `${days}d ${hours}h ${minutes}m`;
        } else if (hours > 0) {
            return `${hours}h ${minutes}m`;
        } else {
            return `${minutes}m`;
        }
    }

    showError(message) {
        console.error('Dashboard error:', message);
        // Could show a toast notification here
    }

    destroy() {
        this.stopMetricsUpdates();

        // Destroy charts
        Object.values(this.charts).forEach(chart => {
            if (chart && typeof chart.destroy === 'function') {
                chart.destroy();
            }
        });

        this.charts = {};
    }
}

// Global dashboard manager instance
const Dashboard = new DashboardManager();