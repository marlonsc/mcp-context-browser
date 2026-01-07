//! Authentication and authorization for admin interface

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::admin::models::{ApiResponse, LoginRequest, LoginResponse, UserInfo};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // username
    pub role: String,     // user role
    pub exp: usize,       // expiration timestamp
    pub iat: usize,       // issued at timestamp
}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
    jwt_expiration: u64,
    admin_username: String,
    admin_password_hash: String,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(
        jwt_secret: String,
        jwt_expiration: u64,
        admin_username: String,
        admin_password_hash: String,
    ) -> Self {
        Self {
            jwt_secret,
            jwt_expiration,
            admin_username,
            admin_password_hash,
        }
    }

    /// Authenticate user credentials
    pub fn authenticate(&self, username: &str, password: &str) -> Result<UserInfo, String> {
        // Simple authentication - in production, use proper password hashing
        if username == self.admin_username && self.verify_password(password) {
            Ok(UserInfo {
                username: username.to_string(),
                role: "admin".to_string(),
            })
        } else {
            Err("Invalid credentials".to_string())
        }
    }

    /// Verify password (simple implementation - use proper hashing in production)
    fn verify_password(&self, password: &str) -> bool {
        // For demo purposes - in production, use bcrypt or argon2
        // For now, just check if password matches "admin" for development
        password == "admin"
    }

    /// Generate JWT token
    pub fn generate_token(&self, user: &UserInfo) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs() as usize;

        let expiration = now + self.jwt_expiration as usize;

        let claims = Claims {
            sub: user.username.clone(),
            role: user.role.clone(),
            exp: expiration,
            iat: now,
        };

        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| format!("Token generation error: {}", e))
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| format!("Token validation error: {}", e))?;

        Ok(token_data.claims)
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<crate::admin::models::AdminState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
        let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, "Missing authentication token").into_response());
        }
    };

    // For now, skip validation in development
    // TODO: Implement proper JWT validation
    let claims = match state.admin_api.config().jwt_secret.as_str() {
        "default-jwt-secret-change-in-production" => {
            // Development mode - accept any token
            Ok(crate::admin::auth::Claims {
                sub: "admin".to_string(),
                role: "admin".to_string(),
                exp: 0,
                iat: 0,
            })
        }
        _ => {
            // Production mode - validate token
            let auth_service = AuthService::new(
                state.admin_api.config().jwt_secret.clone(),
                state.admin_api.config().jwt_expiration,
                state.admin_api.config().username.clone(),
                state.admin_api.config().password_hash.clone(),
            );

            auth_service.validate_token(token)
        }
    };

        match claims {
        Ok(claims) => {
            // Add user info to request extensions
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => {
            Ok((StatusCode::UNAUTHORIZED, "Invalid authentication token").into_response())
        }
    }
}

/// Login handler
pub async fn login_handler(
    State(state): State<crate::admin::models::AdminState>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    let auth_service = AuthService::new(
        state.admin_api.config().jwt_secret.clone(),
        state.admin_api.config().jwt_expiration,
        state.admin_api.config().username.clone(),
        state.admin_api.config().password_hash.clone(),
    );

    match auth_service.authenticate(&login_req.username, &login_req.password) {
        Ok(user) => {
            match auth_service.generate_token(&user) {
                Ok(token) => {
                    let expires_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() + state.admin_api.config().jwt_expiration;

                    let response = LoginResponse {
                        token,
                        expires_at,
                        user,
                    };

                    Ok(Json(ApiResponse::success(response)))
                }
                Err(e) => Ok(Json(ApiResponse::error(format!("Token generation failed: {}", e)))),
            }
        }
        Err(e) => Ok(Json(ApiResponse::error(e))),
    }
}