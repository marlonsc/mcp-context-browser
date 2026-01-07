// MCP Context Browser Admin Interface

class AdminApp {
    constructor() {
        this.currentPage = 'dashboard';
        this.authToken = null;
        this.init();
    }

    init() {
        this.checkAuth();
        this.setupEventListeners();
        this.loadDashboard();
    }

    checkAuth() {
        const token = localStorage.getItem('admin_token');
        if (token) {
            this.authToken = token;
            this.showApp();
        } else {
            this.showLogin();
        }
    }

    showLogin() {
        document.getElementById('login-modal').classList.add('show');
        document.getElementById('app').style.display = 'none';
    }

    showApp() {
        document.getElementById('login-modal').classList.remove('show');
        document.getElementById('app').style.display = 'block';
        this.navigateToPage(this.currentPage);
    }

    setupEventListeners() {
        // Login form
        document.getElementById('login-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.handleLogin();
        });

        // Close login modal
        document.querySelector('.close').addEventListener('click', () => {
            this.showLogin();
        });

        // Navigation
        document.querySelectorAll('.nav-link').forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const page = e.target.dataset.page;
                this.navigateToPage(page);
            });
        });

        // Logout
        document.getElementById('logout-btn').addEventListener('click', () => {
            this.logout();
        });

        // Refresh buttons
        document.getElementById('refresh-dashboard').addEventListener('click', () => {
            this.loadDashboard();
        });

        document.getElementById('refresh-providers').addEventListener('click', () => {
            this.loadProviders();
        });

        document.getElementById('refresh-indexes').addEventListener('click', () => {
            this.loadIndexes();
        });
    }

    async handleLogin() {
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;

        try {
            const response = await API.login(username, password);
            if (response.success) {
                this.authToken = response.data.token;
                localStorage.setItem('admin_token', this.authToken);
                this.showApp();
                document.getElementById('login-error').style.display = 'none';
            } else {
                this.showLoginError(response.error || 'Login failed');
            }
        } catch (error) {
            this.showLoginError('Network error. Please try again.');
        }
    }

    showLoginError(message) {
        const errorDiv = document.getElementById('login-error');
        errorDiv.textContent = message;
        errorDiv.style.display = 'block';
    }

    navigateToPage(page) {
        // Update navigation
        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.remove('active');
        });
        document.querySelector(`[data-page="${page}"]`).classList.add('active');

        // Update pages
        document.querySelectorAll('.page').forEach(pageEl => {
            pageEl.classList.remove('active');
        });
        document.getElementById(`${page}-page`).classList.add('active');

        this.currentPage = page;

        // Load page data
        switch (page) {
            case 'dashboard':
                this.loadDashboard();
                break;
            case 'providers':
                this.loadProviders();
                break;
            case 'indexes':
                this.loadIndexes();
                break;
            case 'config':
                this.loadConfig();
                break;
            case 'search':
                // Search page doesn't need initial load
                break;
        }
    }

    async loadDashboard() {
        try {
            const [healthResponse, providersResponse, statusResponse] = await Promise.all([
                API.getHealth(),
                API.getProviders(),
                API.getStatus()
            ]);

            if (healthResponse.success) {
                this.updateHealthStatus(healthResponse.data);
            }

            if (providersResponse.success) {
                this.updateProvidersStatus(providersResponse.data);
            }

            if (statusResponse.success) {
                this.updateSystemStatus(statusResponse.data);
            }

            // Load charts
            this.loadCharts();

        } catch (error) {
            console.error('Failed to load dashboard:', error);
            this.showError('Failed to load dashboard data');
        }
    }

    updateHealthStatus(health) {
        document.getElementById('uptime').textContent = this.formatUptime(health.uptime);
        document.getElementById('pid').textContent = health.pid;

        const statusIndicator = document.querySelector('.status-indicator');
        statusIndicator.textContent = health.status;
        statusIndicator.className = 'status-indicator status-healthy';
    }

    updateProvidersStatus(providers) {
        const activeProviders = providers.filter(p => p.status === 'active').length;
        document.getElementById('active-providers').textContent = activeProviders;
        document.getElementById('total-providers').textContent = providers.length;
    }

    updateSystemStatus(status) {
        // Update additional status information
        const indexes = status.indexes || {};
        document.getElementById('active-indexes').textContent = indexes.active || 0;
        document.getElementById('total-documents').textContent = indexes.total_docs || 0;
    }

    async loadProviders() {
        try {
            const response = await API.getProviders();
            if (response.success) {
                this.renderProviders(response.data);
            }
        } catch (error) {
            console.error('Failed to load providers:', error);
            this.showError('Failed to load providers');
        }
    }

    renderProviders(providers) {
        const container = document.querySelector('.providers-grid');
        container.innerHTML = '';

        if (providers.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>No providers configured</h3>
                    <p>Add your first provider to get started with semantic search.</p>
                </div>
            `;
            return;
        }

        providers.forEach(provider => {
            const card = this.createProviderCard(provider);
            container.appendChild(card);
        });
    }

    createProviderCard(provider) {
        const card = document.createElement('div');
        card.className = `provider-card ${provider.status}`;

        card.innerHTML = `
            <div class="provider-header">
                <div>
                    <div class="provider-name">${provider.name}</div>
                    <div class="provider-type">${provider.provider_type}</div>
                </div>
                <span class="provider-status ${provider.status}">${provider.status}</span>
            </div>
            <div class="provider-actions">
                <button class="btn btn-secondary btn-sm">Configure</button>
                <button class="btn btn-danger btn-sm">Remove</button>
            </div>
        `;

        return card;
    }

    async loadIndexes() {
        try {
            const response = await API.getIndexes();
            if (response.success) {
                this.renderIndexes(response.data);
            }
        } catch (error) {
            console.error('Failed to load indexes:', error);
            this.showError('Failed to load indexes');
        }
    }

    renderIndexes(indexes) {
        const container = document.querySelector('.indexes-list');
        container.innerHTML = '';

        if (indexes.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>No indexes found</h3>
                    <p>Create your first index to start indexing codebases.</p>
                </div>
            `;
            return;
        }

        indexes.forEach(index => {
            const card = this.createIndexCard(index);
            container.appendChild(card);
        });
    }

    createIndexCard(index) {
        const card = document.createElement('div');
        card.className = 'index-card';

        card.innerHTML = `
            <div class="index-header">
                <div class="index-name">${index.name}</div>
                <span class="index-status ${index.status}">${index.status}</span>
            </div>
            <div class="index-metrics">
                <div class="index-metric">
                    <div class="index-metric-value">${index.document_count.toLocaleString()}</div>
                    <div class="index-metric-label">Documents</div>
                </div>
                <div class="index-metric">
                    <div class="index-metric-value">${this.formatDate(index.created_at)}</div>
                    <div class="index-metric-label">Created</div>
                </div>
                <div class="index-metric">
                    <div class="index-metric-value">${this.formatDate(index.updated_at)}</div>
                    <div class="index-metric-label">Updated</div>
                </div>
            </div>
            <div class="index-actions">
                <button class="btn btn-primary btn-sm">Rebuild</button>
                <button class="btn btn-secondary btn-sm">Status</button>
                <button class="btn btn-danger btn-sm">Clear</button>
            </div>
        `;

        return card;
    }

    async loadConfig() {
        try {
            const response = await API.getConfig();
            if (response.success) {
                this.renderConfig(response.data);
            }
        } catch (error) {
            console.error('Failed to load config:', error);
            this.showError('Failed to load configuration');
        }
    }

    renderConfig(config) {
        const container = document.querySelector('.config-sections');
        container.innerHTML = '';

        // Providers section
        const providersSection = this.createConfigSection('Providers', config.providers || []);
        container.appendChild(providersSection);

        // Indexing section
        const indexingSection = this.createConfigSection('Indexing', config.indexing || {});
        container.appendChild(indexingSection);

        // Security section
        const securitySection = this.createConfigSection('Security', config.security || {});
        container.appendChild(securitySection);

        // Metrics section
        const metricsSection = this.createConfigSection('Metrics', config.metrics || {});
        container.appendChild(metricsSection);
    }

    createConfigSection(title, data) {
        const section = document.createElement('div');
        section.className = 'config-section';

        section.innerHTML = `
            <h3>${title}</h3>
            <div class="config-form">
                <!-- Config fields will be populated based on data type -->
            </div>
        `;

        return section;
    }

    async loadCharts() {
        // Load Chart.js if not already loaded
        if (typeof Chart === 'undefined') {
            await this.loadChartJS();
        }

        // Initialize charts
        this.initMetricsChart();
        this.initQueryChart();
    }

    async loadChartJS() {
        return new Promise((resolve) => {
            const script = document.createElement('script');
            script.src = 'https://cdn.jsdelivr.net/npm/chart.js';
            script.onload = resolve;
            document.head.appendChild(script);
        });
    }

    initMetricsChart() {
        const ctx = document.getElementById('metrics-chart');
        if (!ctx) return;

        new Chart(ctx, {
            type: 'line',
            data: {
                labels: ['1m', '2m', '3m', '4m', '5m', '6m'],
                datasets: [{
                    label: 'CPU Usage (%)',
                    data: [12, 19, 15, 25, 22, 30],
                    borderColor: '#2563eb',
                    backgroundColor: 'rgba(37, 99, 235, 0.1)',
                    tension: 0.4
                }, {
                    label: 'Memory Usage (%)',
                    data: [45, 52, 48, 61, 55, 67],
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

    initQueryChart() {
        const ctx = document.getElementById('query-chart');
        if (!ctx) return;

        new Chart(ctx, {
            type: 'bar',
            data: {
                labels: ['Search', 'Index', 'Status', 'Clear'],
                datasets: [{
                    label: 'Requests per minute',
                    data: [12, 3, 8, 1],
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

    logout() {
        localStorage.removeItem('admin_token');
        this.authToken = null;
        this.showLogin();
    }

    showError(message) {
        // Simple error notification - could be enhanced with a toast system
        alert(`Error: ${message}`);
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

    formatDate(timestamp) {
        const date = new Date(timestamp * 1000);
        return date.toLocaleDateString();
    }
}

// Initialize the app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.adminApp = new AdminApp();
});