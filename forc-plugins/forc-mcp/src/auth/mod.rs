pub mod service;
pub mod storage;

pub use service::{
    create_api_key, delete_api_key, get_api_key, import_api_keys, list_api_keys, AuthManager,
};
pub use storage::{ApiKeyStorage, FileStorage, InMemoryStorage};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Auth-specific error types
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Rate limit exceeded: too many requests per minute")]
    RateLimitExceededPerMinute,

    #[error("Rate limit exceeded: too many requests per day")]
    RateLimitExceededPerDay,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

/// User role for API key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    User,
}

/// API Key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub role: Role,
    pub requests_today: u32,
    pub requests_this_minute: u32,
    pub last_request_minute: Option<DateTime<Utc>>,
    pub last_request_day: Option<DateTime<Utc>>,
}

/// API Key creation response
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: String, // Only returned on creation
}

/// Generic error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Auth configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    /// Require API key for all requests
    pub api_keys_only: bool,
    /// Path to persist API keys (default: in-memory only)
    pub api_keys_file: Option<String>,
    /// Pre-configured admin API key
    pub admin_api_key: Option<String>,
    pub public_rate_limit_per_minute: u32,
    pub public_rate_limit_per_day: u32,
    pub api_key_rate_limit_per_minute: u32,
    pub api_key_rate_limit_per_day: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys_only: false,
            api_keys_file: None,
            admin_api_key: None,
            public_rate_limit_per_minute: 10,
            public_rate_limit_per_day: 1_000,
            api_key_rate_limit_per_minute: 120,
            api_key_rate_limit_per_day: 10_000,
        }
    }
}

/// Generate a new API key
pub fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(random_bytes);
    let result = hasher.finalize();
    format!("mcp_{}", hex::encode(&result[..16])) // Use first 16 bytes for a shorter key
}

/// Extract API key from X-API-Key header
pub fn extract_api_key(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .map(|key| key.to_string())
}

/// API key data for import with optional usage stats
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportApiKey {
    pub id: String,
    pub role: Role,
    #[serde(default)]
    pub requests_today: u32,
    #[serde(default)]
    pub requests_this_minute: u32,
    pub last_request_minute: Option<DateTime<Utc>>,
    pub last_request_day: Option<DateTime<Utc>>,
}

/// Data structure for importing API keys
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRequest {
    pub api_keys: Vec<ImportApiKey>,
    /// If true, clear all existing keys before importing. If false (default), merge with existing keys.
    #[serde(default)]
    pub clear_existing: bool,
}
