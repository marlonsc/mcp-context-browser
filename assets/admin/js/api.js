// API client for MCP Context Browser Admin Interface

class APIClient {
    constructor() {
        this.baseURL = window.location.origin;
        this.authToken = null;
    }

    setAuthToken(token) {
        this.authToken = token;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseURL}${endpoint}`;
        const config = {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        };

        // Add auth token if available
        if (this.authToken) {
            config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // Convert body to JSON if it's an object
        if (config.body && typeof config.body === 'object') {
            config.body = JSON.stringify(config.body);
        }

        try {
            const response = await fetch(url, config);

            // Handle 401 (Unauthorized)
            if (response.status === 401) {
                // Token might be expired, redirect to login
                if (window.adminApp) {
                    window.adminApp.logout();
                }
                throw new Error('Authentication required');
            }

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || `HTTP ${response.status}: ${response.statusText}`);
            }

            return data;
        } catch (error) {
            console.error(`API request failed: ${endpoint}`, error);
            throw error;
        }
    }

    // Authentication
    async login(username, password) {
        return this.request('/admin/auth/login', {
            method: 'POST',
            body: { username, password }
        });
    }

    // System endpoints
    async getHealth() {
        return this.request('/api/health');
    }

    async getStatus() {
        return this.request('/admin/status');
    }

    // Configuration endpoints
    async getConfig() {
        return this.request('/admin/config');
    }

    async updateConfig(config) {
        return this.request('/admin/config', {
            method: 'PUT',
            body: config
        });
    }

    // Provider endpoints
    async getProviders() {
        return this.request('/admin/providers');
    }

    async addProvider(providerConfig) {
        return this.request('/admin/providers', {
            method: 'POST',
            body: providerConfig
        });
    }

    async removeProvider(providerId) {
        return this.request(`/admin/providers/${providerId}`, {
            method: 'DELETE'
        });
    }

    // Index endpoints
    async getIndexes() {
        return this.request('/admin/indexes');
    }

    async createIndex(indexConfig) {
        return this.request('/admin/indexes', {
            method: 'POST',
            body: indexConfig
        });
    }

    async indexOperation(indexId, operation) {
        return this.request(`/admin/indexes/${indexId}/operations`, {
            method: 'POST',
            body: operation
        });
    }

    // Search endpoints
    async search(query, options = {}) {
        const params = new URLSearchParams({
            q: query,
            ...options
        });

        return this.request(`/admin/search?${params}`);
    }

    // Metrics endpoints (existing)
    async getMetrics() {
        return this.request('/api/context/metrics');
    }

    async getCpuMetrics() {
        return this.request('/api/context/metrics/cpu');
    }

    async getMemoryMetrics() {
        return this.request('/api/context/metrics/memory');
    }

    async getQueryMetrics() {
        return this.request('/api/context/metrics/queries');
    }

    async getCacheMetrics() {
        return this.request('/api/context/metrics/cache');
    }

    async getSystemStatus() {
        return this.request('/api/context/status');
    }
}

// Global API instance
const API = new APIClient();

// Set auth token from localStorage if available
const savedToken = localStorage.getItem('admin_token');
if (savedToken) {
    API.setAuthToken(savedToken);
}