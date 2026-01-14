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

use super::models::{ApiResponse, LoginRequest, LoginResponse, UserInfo};
use crate::infrastructure::utils::TimeUtils;

/// Cookie name for JWT token
pub const AUTH_COOKIE_NAME: &str = "mcp_admin_token";

/// JWT claims structure for authentication tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (username) of the token
    pub sub: String,
    /// User role for authorization
    pub role: String,
    /// Expiration timestamp (Unix epoch)
    pub exp: usize,
    /// Issued at timestamp (Unix epoch)
    pub iat: usize,
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
    ///
    /// The admin_password can be either:
    /// - A plaintext password (will be hashed with bcrypt)
    /// - An already-hashed password starting with "$argon2" or "$2" (used as-is)
    pub fn new(
        jwt_secret: String,
        jwt_expiration: u64,
        admin_username: String,
        admin_password: String,
    ) -> Result<Self, String> {
        // Check if password is already a hash (Argon2 or bcrypt format)
        let admin_password_hash =
            if admin_password.starts_with("$argon2") || admin_password.starts_with("$2") {
                // Already a hash, use as-is
                admin_password
            } else {
                // Plaintext password, hash it with bcrypt
                hash(&admin_password, DEFAULT_COST)
                    .map_err(|e| format!("Failed to hash password: {}", e))?
            };

        Ok(Self {
            jwt_secret,
            jwt_expiration,
            admin_username,
            admin_password_hash,
        })
    }

    /// Authenticate user credentials
    pub fn authenticate(&self, username: &str, password: &str) -> Result<UserInfo, String> {
        tracing::debug!(
            "[AUTH] Authenticating: input_username={}, expected_username={}, username_match={}",
            username,
            self.admin_username,
            username == self.admin_username
        );
        tracing::debug!(
            "[AUTH] Hash info: starts_with_argon2={}, starts_with_bcrypt={}, hash_len={}",
            self.admin_password_hash.starts_with("$argon2"),
            self.admin_password_hash.starts_with("$2"),
            self.admin_password_hash.len()
        );

        let password_valid = self.verify_password(password);
        tracing::debug!("[AUTH] Password verification result: {}", password_valid);

        if username == self.admin_username && password_valid {
            Ok(UserInfo {
                username: username.to_string(),
                role: "admin".to_string(),
            })
        } else {
            Err("Invalid credentials".to_string())
        }
    }

    /// Verify password against stored hash (supports both Argon2 and bcrypt)
    fn verify_password(&self, password: &str) -> bool {
        if self.admin_password_hash.starts_with("$argon2") {
            // Argon2 hash - use the infrastructure auth password module
            crate::infrastructure::auth::password::verify_password(
                password,
                &self.admin_password_hash,
            )
            .unwrap_or(false)
        } else {
            // bcrypt hash
            verify(password, &self.admin_password_hash).unwrap_or(false)
        }
    }

    /// Generate JWT token
    pub fn generate_token(&self, user: &UserInfo) -> Result<String, String> {
        let now = TimeUtils::now_unix_secs() as usize;
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
/// Validates JWT tokens using the real AuthService from DI container.
/// All requests must have a valid Bearer token in the Authorization header.
pub async fn auth_middleware(
    State(state): State<super::models::AdminState>,
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

    // Use the real AuthService from DI container
    match state.auth_service.validate_token(token) {
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
    State(state): State<super::models::AdminState>,
    Json(login_req): Json<LoginRequest>,
) -> Response {
    // Use the real AuthService from DI container
    match state.auth_service.authenticate(&login_req.username, &login_req.password) {
        Ok(token) => {
            // For backward compatibility, create a UserInfo from the token claims
            // This is needed because the response format expects UserInfo
            match state.auth_service.validate_token(&token) {
                Ok(claims) => {
                    let user = UserInfo {
                        username: claims.sub.clone(),
                        role: format!("{:?}", claims.role).to_lowercase(),
                    };

                    let jwt_expiration = 3600; // Default 1 hour, could be configurable
                    let expires_at = TimeUtils::now_unix_secs() + jwt_expiration;

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
                    "Token validation failed: {}",
                    e
                )))
                .into_response(),
            }
        }
        Err(e) => Json(ApiResponse::<LoginResponse>::error(e.to_string())).into_response(),
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
    State(state): State<super::models::AdminState>,
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

    // Use the real AuthService from DI container

    // Validate token with proper cryptographic signature verification
    match state.auth_service.validate_token(&token) {
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
///
/// By default, the cookie includes the `Secure` flag which requires HTTPS.
/// For development environments (when MCP_DEV_MODE is set), the Secure flag
/// is omitted to allow HTTP connections.
pub fn create_auth_cookie(token: &str, expires_in_secs: u64) -> String {
    // In development mode, omit Secure flag to allow HTTP
    // In production (default), require HTTPS with Secure flag
    let secure_flag = if std::env::var("MCP_DEV_MODE").is_ok() {
        "" // No Secure flag in dev mode (allows HTTP)
    } else {
        "; Secure" // Require HTTPS in production
    };

    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        AUTH_COOKIE_NAME, token, expires_in_secs, secure_flag
    )
}
