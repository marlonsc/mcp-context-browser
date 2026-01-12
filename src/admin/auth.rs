//! Authentication and authorization for admin interface

use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::admin::models::{ApiResponse, LoginRequest, LoginResponse, UserInfo};

/// Cookie name for JWT token
pub const AUTH_COOKIE_NAME: &str = "mcp_admin_token";

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // username
    pub role: String, // user role
    pub exp: usize,   // expiration timestamp
    pub iat: usize,   // issued at timestamp
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
        admin_password: String,
    ) -> Result<Self, String> {
        let admin_password_hash = hash(admin_password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        Ok(Self {
            jwt_secret,
            jwt_expiration,
            admin_username,
            admin_password_hash,
        })
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

    /// Verify password using bcrypt
    fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.admin_password_hash).unwrap_or(false)
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

// Note: validate_token_simple was removed - it was a security vulnerability that accepted
// any non-empty token. Use validate_token() for all JWT validation which properly verifies
// the cryptographic signature and expiration.

/// Authentication middleware
///
/// Validates JWT tokens using proper cryptographic signature verification.
/// All requests must have a valid Bearer token in the Authorization header.
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

    // Create auth service with proper configuration
    let auth_service = match AuthService::new(
        state.admin_api.config().jwt_secret.clone(),
        state.admin_api.config().jwt_expiration,
        state.admin_api.config().username.clone(),
        state.admin_api.config().password.clone(),
    ) {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Auth service initialization failed: {}", e);
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication service unavailable",
            )
                .into_response());
        }
    };

    // Validate token with proper cryptographic signature verification
    match auth_service.validate_token(token) {
        Ok(claims) => {
            // Add user info to request extensions
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::debug!("Token validation failed: {}", e);
            Ok((StatusCode::UNAUTHORIZED, "Invalid authentication token").into_response())
        }
    }
}

/// Login handler
///
/// Authenticates user and returns JWT token in both JSON response and Set-Cookie header.
/// The cookie allows web pages to be authenticated server-side.
pub async fn login_handler(
    State(state): State<crate::admin::models::AdminState>,
    Json(login_req): Json<LoginRequest>,
) -> Response {
    let auth_service = match AuthService::new(
        state.admin_api.config().jwt_secret.clone(),
        state.admin_api.config().jwt_expiration,
        state.admin_api.config().username.clone(),
        state.admin_api.config().password.clone(),
    ) {
        Ok(service) => service,
        Err(e) => {
            return Json(ApiResponse::<LoginResponse>::error(format!(
                "Auth service initialization failed: {}",
                e
            )))
            .into_response();
        }
    };

    match auth_service.authenticate(&login_req.username, &login_req.password) {
        Ok(user) => match auth_service.generate_token(&user) {
            Ok(token) => {
                let jwt_expiration = state.admin_api.config().jwt_expiration;
                let expires_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    + jwt_expiration;

                let response = LoginResponse {
                    token: token.clone(),
                    expires_at,
                    user,
                };

                // Return JSON response with Set-Cookie header
                let cookie_value = create_auth_cookie(&token, jwt_expiration);

                (
                    [(header::SET_COOKIE, cookie_value)],
                    Json(ApiResponse::success(response)),
                )
                    .into_response()
            }
            Err(e) => Json(ApiResponse::<LoginResponse>::error(format!(
                "Token generation failed: {}",
                e
            )))
            .into_response(),
        },
        Err(e) => Json(ApiResponse::<LoginResponse>::error(e)).into_response(),
    }
}

/// Logout handler
pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    // Clear the auth cookie
    let jar = jar.remove(AUTH_COOKIE_NAME);

    (
        jar,
        Json(ApiResponse::success("Logged out successfully".to_string())),
    )
}

/// Web page authentication middleware
///
/// Checks for JWT token in cookie and redirects to login page if not present or invalid.
/// This middleware is for protecting web pages (HTML), not API endpoints.
pub async fn web_auth_middleware(
    State(state): State<crate::admin::models::AdminState>,
    jar: CookieJar,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, Response> {
    // Extract token from cookie
    let token: Option<String> = jar
        .get(AUTH_COOKIE_NAME)
        .map(|cookie: &axum_extra::extract::cookie::Cookie<'_>| cookie.value().to_string());

    let token: String = match token {
        Some(t) => {
            if t.is_empty() {
                // Empty token - redirect to login
                return Err(Redirect::to("/login").into_response());
            }
            t
        }
        None => {
            // No cookie - redirect to login
            return Err(Redirect::to("/login").into_response());
        }
    };

    // Create auth service with proper configuration
    let auth_service = match AuthService::new(
        state.admin_api.config().jwt_secret.clone(),
        state.admin_api.config().jwt_expiration,
        state.admin_api.config().username.clone(),
        state.admin_api.config().password.clone(),
    ) {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Auth service initialization failed: {}", e);
            return Err(Redirect::to("/login").into_response());
        }
    };

    // Validate token with proper cryptographic signature verification
    match auth_service.validate_token(&token) {
        Ok(claims) => {
            // Add user info to request extensions for downstream handlers
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::debug!("Token validation failed for web page: {}", e);
            // Invalid token - redirect to login
            Err(Redirect::to("/login").into_response())
        }
    }
}

/// Create a Set-Cookie header value for the JWT token
pub fn create_auth_cookie(token: &str, expires_in_secs: u64) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        AUTH_COOKIE_NAME, token, expires_in_secs
    )
}
