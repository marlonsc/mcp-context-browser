// Authentication management for admin interface

class AuthManager {
    constructor() {
        this.token = null;
        this.user = null;
        this.init();
    }

    init() {
        // Check for existing session
        const token = localStorage.getItem('admin_token');
        const user = localStorage.getItem('admin_user');

        if (token && user) {
            try {
                this.token = token;
                this.user = JSON.parse(user);
                API.setAuthToken(token);

                // Verify token is still valid
                this.verifyToken();
            } catch (error) {
                console.error('Failed to restore session:', error);
                this.clearSession();
            }
        }
    }

    async login(username, password) {
        try {
            const response = await API.login(username, password);

            if (response.success) {
                this.token = response.data.token;
                this.user = response.data.user;

                // Store session
                localStorage.setItem('admin_token', this.token);
                localStorage.setItem('admin_user', JSON.stringify(this.user));

                // Set token in API client
                API.setAuthToken(this.token);

                return { success: true };
            } else {
                return { success: false, error: response.error };
            }
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    logout() {
        this.clearSession();
        // Redirect to login
        window.location.reload();
    }

    clearSession() {
        this.token = null;
        this.user = null;
        localStorage.removeItem('admin_token');
        localStorage.removeItem('admin_user');
        API.setAuthToken(null);
    }

    async verifyToken() {
        try {
            // Try to make a request that requires authentication
            await API.getConfig();
            return true;
        } catch (error) {
            if (error.message.includes('Authentication required')) {
                this.clearSession();
                return false;
            }
            // Other errors might be temporary, so we assume token is still valid
            return true;
        }
    }

    isAuthenticated() {
        return this.token !== null && this.user !== null;
    }

    getUser() {
        return this.user;
    }

    getToken() {
        return this.token;
    }
}

// Global auth manager instance
const Auth = new AuthManager();