// Frontend Configuration
// This file reads environment variables and provides configuration for the frontend
const CONFIG = {
    // Keycloak Configuration - using exact values from .env
    KEYCLOAK_URL: 'http://localhost:8080',
    KEYCLOAK_REALM: 'upload-realm',
    CLIENT_ID: 'upload-client',
//    
    // Backend Configuration - using exact values from .env
    BACKEND_URL: 'http://localhost:3000',
    
    // Frontend Configuration - using exact values from .env
    FRONTEND_URL: 'http://localhost:8000',
    
    // API Endpoints
    getKeycloakAuthUrl() {
        return `${this.KEYCLOAK_URL}/realms/${this.KEYCLOAK_REALM}/protocol/openid-connect/auth`;
    },
    
    getKeycloakTokenUrl() {
        return `${this.KEYCLOAK_URL}/realms/${this.KEYCLOAK_REALM}/protocol/openid-connect/token`;
    },
    
    getUploadUrl() {
        return `${this.BACKEND_URL}/api/upload`;
    },
    
    getHealthUrl() {
        return `${this.BACKEND_URL}/health`;
    }
};

// Configuration is now hardcoded from .env values for simplicity
// All values are taken directly from the .env file

// Export for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
    module.exports = CONFIG;
} else {
    window.CONFIG = CONFIG;
}
