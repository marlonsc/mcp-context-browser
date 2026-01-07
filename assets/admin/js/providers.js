// Provider management for admin interface

class ProvidersManager {
    constructor() {
        this.providers = [];
        this.init();
    }

    init() {
        // Setup event listeners for provider actions
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Add provider button
        const addBtn = document.getElementById('add-provider-btn');
        if (addBtn) {
            addBtn.addEventListener('click', () => {
                this.showAddProviderModal();
            });
        }

        // Provider action buttons (delegated)
        document.addEventListener('click', (e) => {
            if (e.target.matches('.provider-card .btn')) {
                const action = e.target.textContent.toLowerCase();
                const card = e.target.closest('.provider-card');
                const providerId = card.dataset.providerId;

                switch (action) {
                    case 'configure':
                        this.configureProvider(providerId);
                        break;
                    case 'remove':
                        this.removeProvider(providerId);
                        break;
                }
            }
        });
    }

    async loadProviders() {
        try {
            const response = await API.getProviders();
            if (response.success) {
                this.providers = response.data;
                this.renderProviders();
            }
        } catch (error) {
            console.error('Failed to load providers:', error);
            this.showError('Failed to load providers');
        }
    }

    renderProviders() {
        const container = document.querySelector('.providers-grid');
        if (!container) return;

        container.innerHTML = '';

        if (this.providers.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>No providers configured</h3>
                    <p>Add your first provider to get started with semantic search.</p>
                    <button id="add-first-provider" class="btn btn-primary">Add Provider</button>
                </div>
            `;

            document.getElementById('add-first-provider')?.addEventListener('click', () => {
                this.showAddProviderModal();
            });

            return;
        }

        this.providers.forEach(provider => {
            const card = this.createProviderCard(provider);
            container.appendChild(card);
        });
    }

    createProviderCard(provider) {
        const card = document.createElement('div');
        card.className = `provider-card ${provider.status}`;
        card.dataset.providerId = provider.id;

        const configSummary = this.getConfigSummary(provider.config);

        card.innerHTML = `
            <div class="provider-header">
                <div>
                    <div class="provider-name">${provider.name}</div>
                    <div class="provider-type">${provider.provider_type}</div>
                </div>
                <span class="provider-status ${provider.status}">${provider.status}</span>
            </div>
            <div class="provider-config">
                <small>${configSummary}</small>
            </div>
            <div class="provider-actions">
                <button class="btn btn-secondary btn-sm">Configure</button>
                <button class="btn btn-danger btn-sm">Remove</button>
            </div>
        `;

        return card;
    }

    getConfigSummary(config) {
        if (!config || typeof config !== 'object') {
            return 'No configuration';
        }

        const keys = Object.keys(config);
        if (keys.length === 0) {
            return 'Empty configuration';
        }

        // Show up to 3 config keys
        const summary = keys.slice(0, 3).map(key => {
            const value = config[key];
            if (typeof value === 'string' && value.length > 20) {
                return `${key}: ${value.substring(0, 20)}...`;
            }
            return `${key}: ${value}`;
        }).join(', ');

        if (keys.length > 3) {
            return `${summary} (+${keys.length - 3} more)`;
        }

        return summary;
    }

    showAddProviderModal() {
        // Create and show add provider modal
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content">
                <div class="modal-header">
                    <h2>Add Provider</h2>
                    <span class="close">&times;</span>
                </div>
                <div class="modal-body">
                    <form id="add-provider-form">
                        <div class="form-group">
                            <label for="provider-type">Provider Type</label>
                            <select id="provider-type" name="provider_type" required>
                                <option value="">Select type...</option>
                                <option value="embedding">Embedding Provider</option>
                                <option value="vector_store">Vector Store Provider</option>
                            </select>
                        </div>
                        <div id="provider-config-fields">
                            <!-- Dynamic fields will be added here -->
                        </div>
                        <div class="form-actions">
                            <button type="submit" class="btn btn-primary">Add Provider</button>
                            <button type="button" class="btn btn-secondary" onclick="this.closest('.modal').remove()">Cancel</button>
                        </div>
                    </form>
                </div>
            </div>
        `;

        document.body.appendChild(modal);
        modal.classList.add('show');

        // Setup form
        this.setupAddProviderForm(modal);

        // Close modal
        modal.querySelector('.close').addEventListener('click', () => {
            modal.remove();
        });
    }

    setupAddProviderForm(modal) {
        const typeSelect = modal.querySelector('#provider-type');
        const configFields = modal.querySelector('#provider-config-fields');
        const form = modal.querySelector('#add-provider-form');

        typeSelect.addEventListener('change', () => {
            this.updateProviderConfigFields(configFields, typeSelect.value);
        });

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.addProvider(form);
            modal.remove();
        });
    }

    updateProviderConfigFields(container, providerType) {
        let fields = '';

        switch (providerType) {
            case 'embedding':
                fields = `
                    <div class="form-group">
                        <label for="provider-name">Provider Name</label>
                        <select id="provider-name" name="name" required>
                            <option value="openai">OpenAI</option>
                            <option value="ollama">Ollama</option>
                            <option value="mock">Mock (Testing)</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="api-key">API Key (for OpenAI)</label>
                        <input type="password" id="api-key" name="api_key">
                    </div>
                    <div class="form-group">
                        <label for="base-url">Base URL</label>
                        <input type="url" id="base-url" name="base_url" placeholder="https://api.openai.com/v1">
                    </div>
                    <div class="form-group">
                        <label for="model">Model</label>
                        <input type="text" id="model" name="model" placeholder="text-embedding-ada-002">
                    </div>
                `;
                break;
            case 'vector_store':
                fields = `
                    <div class="form-group">
                        <label for="provider-name">Provider Name</label>
                        <select id="provider-name" name="name" required>
                            <option value="milvus">Milvus</option>
                            <option value="in-memory">In-Memory (Testing)</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="host">Host</label>
                        <input type="text" id="host" name="host" placeholder="localhost">
                    </div>
                    <div class="form-group">
                        <label for="port">Port</label>
                        <input type="number" id="port" name="port" placeholder="19530">
                    </div>
                `;
                break;
        }

        container.innerHTML = fields;
    }

    async addProvider(form) {
        const formData = new FormData(form);
        const providerData = {
            provider_type: formData.get('provider_type'),
            name: formData.get('name'),
            config: {}
        };

        // Collect config fields
        for (let [key, value] of formData.entries()) {
            if (key !== 'provider_type' && key !== 'name' && value) {
                if (key === 'port') {
                    providerData.config[key] = parseInt(value);
                } else {
                    providerData.config[key] = value;
                }
            }
        }

        try {
            const response = await API.addProvider(providerData);
            if (response.success) {
                this.showSuccess('Provider added successfully');
                this.loadProviders();
            } else {
                this.showError(response.error || 'Failed to add provider');
            }
        } catch (error) {
            this.showError('Failed to add provider: ' + error.message);
        }
    }

    async removeProvider(providerId) {
        if (!confirm('Are you sure you want to remove this provider?')) {
            return;
        }

        try {
            const response = await API.removeProvider(providerId);
            if (response.success) {
                this.showSuccess('Provider removed successfully');
                this.loadProviders();
            } else {
                this.showError(response.error || 'Failed to remove provider');
            }
        } catch (error) {
            this.showError('Failed to remove provider: ' + error.message);
        }
    }

    configureProvider(providerId) {
        // TODO: Implement provider configuration
        this.showError('Provider configuration not yet implemented');
    }

    showError(message) {
        // Simple error notification
        alert('Error: ' + message);
    }

    showSuccess(message) {
        // Simple success notification
        alert('Success: ' + message);
    }
}

// Global providers manager instance
const Providers = new ProvidersManager();