use super::{
    generate_api_key, ApiKey, ApiKeyStorage, AuthConfig, AuthError, CreateApiKeyResponse,
    ErrorResponse, FileStorage, ImportRequest, InMemoryStorage, Role,
};
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};

/// Auth manager that handles API keys
pub struct AuthManager {
    pub config: AuthConfig,
    pub storage: Arc<dyn ApiKeyStorage>,
    pub admin_api_key: String,
}

impl AuthManager {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        let storage: Arc<dyn ApiKeyStorage> = if let Some(file_path) = &config.api_keys_file {
            Arc::new(FileStorage::new(file_path).await?)
        } else {
            Arc::new(InMemoryStorage::new())
        };

        // Use provided admin API key or generate one
        let admin_api_key = config
            .admin_api_key
            .clone()
            .unwrap_or_else(generate_api_key);

        let admin_key_info = ApiKey {
            id: admin_api_key.clone(),
            role: Role::Admin,
            requests_today: 0,
            requests_this_minute: 0,
            last_request_minute: None,
            last_request_day: None,
        };

        storage.create(admin_key_info).await?;

        tracing::info!("Auth Manager initialized");
        tracing::info!("Admin API Key: {}", admin_api_key);

        Ok(Self {
            config,
            storage,
            admin_api_key,
        })
    }

    /// Check and update API key usage, enforcing rate limits
    pub async fn check_and_track_usage(&self, key: &str) -> Result<Option<ApiKey>, AuthError> {
        let api_key = self
            .storage
            .get(key)
            .await
            .map_err(|e| AuthError::StorageError(e.to_string()))?;

        if let Some(mut key_info) = api_key {
            let now = Utc::now();

            // Check if we need to reset counters
            let needs_minute_reset = key_info
                .last_request_minute
                .map(|last| (now - last).num_seconds() >= 60)
                .unwrap_or(true);

            let needs_day_reset = key_info
                .last_request_day
                .map(|last| {
                    let last_date = last.date_naive();
                    let now_date = now.date_naive();
                    last_date != now_date
                })
                .unwrap_or(true);

            // Reset counters if needed
            if needs_minute_reset {
                key_info.requests_this_minute = 0;
                key_info.last_request_minute = Some(now);
            }

            if needs_day_reset {
                key_info.requests_today = 0;
                key_info.last_request_day = Some(now);
            }

            // Check rate limits based on role
            match key_info.role {
                // No rate limits for admin
                Role::Admin => {}
                // Use default rate limits from config
                Role::User => {
                    if key_info.requests_this_minute >= self.config.api_key_rate_limit_per_minute {
                        return Err(AuthError::RateLimitExceededPerMinute);
                    }
                    if key_info.requests_today >= self.config.api_key_rate_limit_per_day {
                        return Err(AuthError::RateLimitExceededPerDay);
                    }
                }
            }

            // Increment counters
            key_info.requests_this_minute += 1;
            key_info.requests_today += 1;

            // Update timestamps
            key_info.last_request_minute = Some(now);
            key_info.last_request_day = Some(now);

            // Update storage
            self.storage
                .update(&key_info.id, key_info.clone())
                .await
                .map_err(|e| AuthError::StorageError(e.to_string()))?;
            Ok(Some(key_info))
        } else {
            Ok(None)
        }
    }

    /// Get usage statistics for an API key
    pub async fn get_usage_stats(&self, key_id: &str) -> Result<(u32, u32)> {
        if let Some(key_info) = self.storage.get(key_id).await? {
            let now = Utc::now();

            // Check if counters need reset
            let needs_minute_reset = key_info
                .last_request_minute
                .map(|last| (now - last).num_seconds() >= 60)
                .unwrap_or(true);

            let needs_day_reset = key_info
                .last_request_day
                .map(|last| {
                    let last_date = last.date_naive();
                    let now_date = now.date_naive();
                    last_date != now_date
                })
                .unwrap_or(true);

            let minute_count = if needs_minute_reset {
                0
            } else {
                key_info.requests_this_minute
            };
            let day_count = if needs_day_reset {
                0
            } else {
                key_info.requests_today
            };

            Ok((minute_count, day_count))
        } else {
            Ok((0, 0))
        }
    }
}

/// Create a new API key
pub async fn create_api_key(
    State(auth_manager): State<Arc<AuthManager>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let key = generate_api_key();
    let key_info = ApiKey {
        id: key.clone(),
        role: Role::User,
        requests_today: 0,
        requests_this_minute: 0,
        last_request_minute: None,
        last_request_day: None,
    };
    match auth_manager.storage.create(key_info).await {
        Ok(_) => {
            let response = CreateApiKeyResponse { api_key: key };
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create API key: {}", e),
            }),
        )),
    }
}

/// List API keys with optional admin inclusion and full details
pub async fn list_api_keys(
    State(auth_manager): State<Arc<AuthManager>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let include_admin = params.get("include_admin").is_some_and(|v| v.to_lowercase() == "true");
    match auth_manager.storage.list().await {
        Ok(keys) => {
            // Return full ApiKey objects
            let filtered_keys: Vec<ApiKey> = keys
                .into_iter()
                .filter(|key| include_admin || key.role != Role::Admin)
                .collect();
            Ok(Json(serde_json::json!({ "api_keys": filtered_keys })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list API keys: {}", e),
            }),
        )),
    }
}

/// Get a specific API key
pub async fn get_api_key(
    State(auth_manager): State<Arc<AuthManager>>,
    Path(key_hash): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match auth_manager.storage.get(&key_hash).await {
        Ok(Some(key)) if key.role != Role::Admin => Ok(Json(key)),
        Ok(Some(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "API key not found".to_string(),
            }),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "API key not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get API key: {}", e),
            }),
        )),
    }
}

/// Delete an API key
pub async fn delete_api_key(
    State(auth_manager): State<Arc<AuthManager>>,
    Path(key_hash): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Check if key exists and is not admin
    match auth_manager.storage.get(&key_hash).await {
        Ok(Some(key)) if key.role == Role::Admin => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Cannot delete admin key".to_string(),
                }),
            ));
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "API key not found".to_string(),
                }),
            ));
        }
        Ok(_) => {} // Key exists and is not admin, proceed
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to check API key: {}", e),
                }),
            ));
        }
    }

    match auth_manager.storage.delete(&key_hash).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete API key: {}", e),
            }),
        )),
    }
}

/// Import API keys with optional merge or replace
pub async fn import_api_keys(
    State(auth_manager): State<Arc<AuthManager>>,
    Json(import_request): Json<ImportRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Convert import keys to full ApiKey, using provided values or defaults
    let full_keys: Vec<ApiKey> = import_request
        .api_keys
        .into_iter()
        .map(|import_key| ApiKey {
            id: import_key.id,
            role: import_key.role,
            requests_today: import_key.requests_today,
            requests_this_minute: import_key.requests_this_minute,
            last_request_minute: import_key.last_request_minute,
            last_request_day: import_key.last_request_day,
        })
        .collect();
    match auth_manager
        .storage
        .create_batch(full_keys, import_request.clear_existing)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to import API keys: {}", e),
            }),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ImportApiKey;

    // Helper function to extract response data from IntoResponse results
    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        result: Result<impl IntoResponse, impl IntoResponse>,
    ) -> Result<(StatusCode, Option<T>)> {
        match result {
            Ok(response) => {
                let response = response.into_response();
                let status = response.status();
                if status == StatusCode::NO_CONTENT {
                    Ok((status, None))
                } else {
                    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
                    let data: T = serde_json::from_slice(&body)?;
                    Ok((status, Some(data)))
                }
            }
            Err(response) => {
                let response = response.into_response();
                let status = response.status();
                Ok((status, None))
            }
        }
    }

    #[tokio::test]
    async fn test_api_key_crud_operations() -> Result<()> {
        // Create an AuthManager with in-memory storage
        let config = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("test-admin-key".to_string()),
            api_keys_file: None, // Use in-memory storage for testing
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager = Arc::new(AuthManager::new(config).await?);
        let state = State(auth_manager.clone());

        // Test CREATE operation
        let create_result = create_api_key(state.clone()).await;
        let (status, create_data) =
            extract_response_data::<CreateApiKeyResponse>(create_result).await?;
        assert_eq!(status, StatusCode::CREATED);
        let api_key = create_data.unwrap().api_key;

        // Test LIST operation
        let list_result = list_api_keys(state.clone(), Query(HashMap::new())).await;
        let (status, list_data) = extract_response_data::<serde_json::Value>(list_result).await?;
        assert_eq!(status, StatusCode::OK);
        let list_response = list_data.unwrap();
        let api_keys = list_response["api_keys"].as_array().unwrap();

        // Should have exactly one key (the one we just created)
        assert!(api_keys.len() == 1);

        // Find our created key in the list
        let found = api_keys.iter().any(|k| {
            // Keys are no longer truncated in list response
            k["id"].as_str().unwrap() == api_key
        });
        assert!(found);

        // Verify admin keys are not in the list
        let admin_found = api_keys
            .iter()
            .any(|k| k["role"].as_str().unwrap() == "Admin");
        assert!(!admin_found);

        // Test GET operation
        let get_result = get_api_key(state.clone(), Path(api_key.clone())).await;
        let (status, get_data) = extract_response_data::<ApiKey>(get_result).await?;
        assert_eq!(status, StatusCode::OK);
        let key_info = get_data.unwrap();
        assert_eq!(key_info.id, api_key);
        assert_eq!(key_info.role, Role::User);
        assert_eq!(key_info.requests_today, 0);

        // Test UPDATE indirectly by tracking usage
        auth_manager.check_and_track_usage(&api_key).await?;

        // Verify usage was tracked by checking again
        let get_result_after_update = get_api_key(state.clone(), Path(api_key.clone())).await;
        let (status, get_data) = extract_response_data::<ApiKey>(get_result_after_update).await?;
        assert_eq!(status, StatusCode::OK);
        let key_info = get_data.unwrap();
        assert_eq!(key_info.requests_today, 1);

        // Test DELETE operation
        let delete_result = delete_api_key(state.clone(), Path(api_key.clone())).await;
        let (status, _) = extract_response_data::<()>(delete_result).await?;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Verify the key was deleted by trying to GET it
        let get_result_after_delete = get_api_key(state.clone(), Path(api_key)).await;
        let (status, _) = extract_response_data::<ApiKey>(get_result_after_delete).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_crud_operations_edge_cases() -> Result<()> {
        // Create an AuthManager with in-memory storage
        let config = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("test-admin-key".to_string()),
            api_keys_file: None, // Use in-memory storage for testing
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager = Arc::new(AuthManager::new(config).await?);
        let state = State(auth_manager.clone());

        // Test getting non-existent key
        let get_nonexistent =
            get_api_key(state.clone(), Path("non-existent-key".to_string())).await;
        let (status, _) = extract_response_data::<ApiKey>(get_nonexistent).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test deleting non-existent key
        let delete_nonexistent =
            delete_api_key(state.clone(), Path("non-existent-key".to_string())).await;
        let (status, _) = extract_response_data::<()>(delete_nonexistent).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test getting empty string key
        let get_empty = get_api_key(state.clone(), Path("".to_string())).await;
        let (status, _) = extract_response_data::<ApiKey>(get_empty).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test deleting empty string key
        let delete_empty = delete_api_key(state.clone(), Path("".to_string())).await;
        let (status, _) = extract_response_data::<()>(delete_empty).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test getting key with special characters
        let get_special = get_api_key(state.clone(), Path("special-@#$%^&*()".to_string())).await;
        let (status, _) = extract_response_data::<ApiKey>(get_special).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test deleting key with special characters
        let delete_special =
            delete_api_key(state.clone(), Path("special-@#$%^&*()".to_string())).await;
        let (status, _) = extract_response_data::<()>(delete_special).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test getting very long key
        let long_key = "a".repeat(1000);
        let get_long = get_api_key(state.clone(), Path(long_key.clone())).await;
        let (status, _) = extract_response_data::<ApiKey>(get_long).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test deleting very long key
        let delete_long = delete_api_key(state.clone(), Path(long_key)).await;
        let (status, _) = extract_response_data::<()>(delete_long).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Admin key edge cases
        let admin_key = auth_manager.admin_api_key.clone();

        // Test that admin keys are not returned by get (should return NOT_FOUND, not the actual key)
        let get_admin = get_api_key(state.clone(), Path(admin_key.clone())).await;
        let (status, data) = extract_response_data::<ApiKey>(get_admin).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(data.is_none());

        // Test that admin keys cannot be deleted (should return FORBIDDEN)
        let delete_admin = delete_api_key(state.clone(), Path(admin_key.clone())).await;
        let (status, _) = extract_response_data::<()>(delete_admin).await?;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Test that admin key variations don't work
        let admin_key_upper = admin_key.to_uppercase();
        let get_admin_upper = get_api_key(state.clone(), Path(admin_key_upper)).await;
        let (status, _) = extract_response_data::<ApiKey>(get_admin_upper).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test LIST with empty storage (only admin key should exist but be filtered out)
        let list_empty = list_api_keys(state.clone(), Query(HashMap::new())).await;
        let (status, list_data) = extract_response_data::<serde_json::Value>(list_empty).await?;
        assert_eq!(status, StatusCode::OK);
        let list_response = list_data.unwrap();
        let api_keys = list_response["api_keys"].as_array().unwrap();

        // Should be empty since admin keys are filtered out
        assert!(api_keys.is_empty());

        // Create a user key and verify admin filtering works
        let create_result = create_api_key(state.clone()).await;
        let (status, create_data) =
            extract_response_data::<CreateApiKeyResponse>(create_result).await?;
        assert_eq!(status, StatusCode::CREATED);
        let user_key = create_data.unwrap().api_key;

        // Verify LIST contains user key but not admin key
        let list_with_user = list_api_keys(state.clone(), Query(HashMap::new())).await;
        let (status, list_data) =
            extract_response_data::<serde_json::Value>(list_with_user).await?;
        assert_eq!(status, StatusCode::OK);
        let list_response = list_data.unwrap();
        let api_keys = list_response["api_keys"].as_array().unwrap();

        // Should have exactly one key (the user key)
        assert_eq!(api_keys.len(), 1);

        // Should contain the user key
        let found_user = api_keys
            .iter()
            .any(|k| k["id"].as_str().unwrap() == user_key);
        assert!(found_user);

        // Should not contain admin key
        let found_admin = api_keys
            .iter()
            .any(|k| k["role"].as_str().unwrap() == "Admin");
        assert!(!found_admin);

        // Test double deletion
        let delete_first = delete_api_key(state.clone(), Path(user_key.clone())).await;
        let (status, _) = extract_response_data::<()>(delete_first).await?;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Try to delete the same key again
        let delete_second = delete_api_key(state.clone(), Path(user_key)).await;
        let (status, _) = extract_response_data::<()>(delete_second).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_persistent_storage() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let storage_file = temp_dir.path().join("api_keys.json");

        // Create AuthManager with file storage
        let config = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("admin-key".to_string()),
            api_keys_file: Some(storage_file.to_string_lossy().to_string()),
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager = Arc::new(AuthManager::new(config).await?);
        let state = State(auth_manager.clone());

        // CREATE: Add a test key via handler
        let create_result = create_api_key(state.clone()).await;
        let (status, create_data) =
            extract_response_data::<CreateApiKeyResponse>(create_result).await?;
        assert_eq!(status, StatusCode::CREATED);
        let test_key = create_data.unwrap().api_key;

        // GET: Retrieve the key via handler
        let get_result = get_api_key(state.clone(), Path(test_key.clone())).await;
        let (status, get_data) = extract_response_data::<ApiKey>(get_result).await?;
        assert_eq!(status, StatusCode::OK);
        let key_info = get_data.unwrap();
        assert_eq!(key_info.id, test_key);
        assert_eq!(key_info.requests_today, 0);

        // LIST: Verify key appears in list via handler
        let list_result = list_api_keys(state.clone(), Query(HashMap::new())).await;
        let (status, list_data) = extract_response_data::<serde_json::Value>(list_result).await?;
        assert_eq!(status, StatusCode::OK);
        let list_response = list_data.unwrap();
        let api_keys = list_response["api_keys"].as_array().unwrap();
        assert!(api_keys
            .iter()
            .any(|k| k["id"].as_str().unwrap() == test_key));

        // UPDATE: Track usage to modify the key
        auth_manager.check_and_track_usage(&test_key).await?;

        // Verify update via handler
        let get_updated = get_api_key(state.clone(), Path(test_key.clone())).await;
        let (status, get_data) = extract_response_data::<ApiKey>(get_updated).await?;
        assert_eq!(status, StatusCode::OK);
        let updated_info = get_data.unwrap();
        assert_eq!(updated_info.requests_today, 1);

        // DELETE: Remove the key via handler
        let delete_result = delete_api_key(state.clone(), Path(test_key.clone())).await;
        let (status, _) = extract_response_data::<()>(delete_result).await?;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Verify deletion via handler
        let get_deleted = get_api_key(state.clone(), Path(test_key.clone())).await;
        let (status, _) = extract_response_data::<ApiKey>(get_deleted).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test persistence: Create new manager with same file
        let config2 = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("admin-key".to_string()),
            api_keys_file: Some(storage_file.to_string_lossy().to_string()),
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager2 = Arc::new(AuthManager::new(config2).await?);
        let state2 = State(auth_manager2.clone());

        // Verify deleted key is still gone via handler
        let get_result2 = get_api_key(state2.clone(), Path(test_key)).await;
        let (status, _) = extract_response_data::<ApiKey>(get_result2).await?;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Verify admin key persisted
        let list_result2 = list_api_keys(state2, Query(HashMap::new())).await;
        let (status, list_data2) = extract_response_data::<serde_json::Value>(list_result2).await?;
        assert_eq!(status, StatusCode::OK);
        let list_response2 = list_data2.unwrap();
        let api_keys2 = list_response2["api_keys"].as_array().unwrap();
        // Should be empty since we deleted the only user key and admin keys are filtered
        assert!(api_keys2.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_rate_limiting() -> Result<()> {
        // Create an AuthManager with in-memory storage
        let config = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("test-admin-key".to_string()),
            api_keys_file: None, // Use in-memory storage for testing
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager = Arc::new(AuthManager::new(config).await?);

        // Create an API key (all user keys have the same rate limits from config)
        let test_api_key = generate_api_key();
        let test_key_info = ApiKey {
            id: test_api_key.clone(),
            role: Role::User,
            requests_today: 0,
            requests_this_minute: 0,
            last_request_minute: None,
            last_request_day: None,
        };
        auth_manager.storage.create(test_key_info).await?;

        // Make requests and verify that usage is tracked
        // Make requests up to the minute limit (60 by default)
        for _ in 1..=auth_manager.config.api_key_rate_limit_per_minute {
            let result = auth_manager.check_and_track_usage(&test_api_key).await?;
            assert!(result.is_some());
        }

        // Verify that rate limiting kicks in after exceeding the limit
        // Next request should fail (exceeds minute limit)
        let result = auth_manager.check_and_track_usage(&test_api_key).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::RateLimitExceededPerMinute));

        // Verify usage stats haven't increased
        let (minute_usage, day_usage) = auth_manager.get_usage_stats(&test_api_key).await?;
        assert_eq!(minute_usage, 60); // Should be at the limit
        assert_eq!(day_usage, 60);

        // Test the reset functionality for time windows
        // Manually update the last_request_minute to simulate time passing
        let mut key_info = auth_manager.storage.get(&test_api_key).await?.unwrap();
        key_info.last_request_minute = Some(Utc::now() - chrono::Duration::seconds(61));
        auth_manager.storage.update(&test_api_key, key_info).await?;

        // Now the request should succeed again (minute counter reset)
        let result = auth_manager.check_and_track_usage(&test_api_key).await?;
        assert!(result.is_some());
        let key_info = result.unwrap();
        assert_eq!(key_info.requests_this_minute, 1); // Reset to 1
        assert_eq!(key_info.requests_today, 61); // Day counter continues

        // Test admin key has no rate limits
        let admin_key = &auth_manager.admin_api_key;

        // Make multiple requests with admin key
        for _ in 1..=5 {
            let result = auth_manager.check_and_track_usage(admin_key).await?;
            assert!(result.is_some());
        }

        // Test day counter reset
        println!("\nTesting day counter reset...");
        let mut key_info = auth_manager.storage.get(&test_api_key).await?.unwrap();
        // Simulate next day by setting last_request_day to yesterday
        key_info.last_request_day = Some(Utc::now() - chrono::Duration::days(1));
        key_info.requests_today = 499; // Close to daily limit
        auth_manager.storage.update(&test_api_key, key_info).await?;

        // Request should succeed with reset day counter
        let result = auth_manager.check_and_track_usage(&test_api_key).await?;
        assert!(result.is_some());
        let key_info = result.unwrap();
        assert_eq!(key_info.requests_today, 1); // Day counter reset
        Ok(())
    }

    #[tokio::test]
    async fn test_export_import_api_keys() -> Result<()> {
        // Create an AuthManager with in-memory storage
        let config = AuthConfig {
            enabled: true,
            api_keys_only: false,
            admin_api_key: Some("test-admin-key".to_string()),
            api_keys_file: None,
            public_rate_limit_per_minute: 100,
            public_rate_limit_per_day: 1000,
            api_key_rate_limit_per_minute: 60,
            api_key_rate_limit_per_day: 500,
        };
        let auth_manager = Arc::new(AuthManager::new(config).await?);

        // Create a user API key
        let create_result = create_api_key(State(auth_manager.clone())).await;
        let (status, create_data) =
            extract_response_data::<CreateApiKeyResponse>(create_result).await?;
        assert_eq!(status, StatusCode::CREATED);
        let user_key = create_data.unwrap().api_key;

        // Export data using list handler with full details - should have 2 keys (1 user + 1 admin)
        let mut export_params = HashMap::new();
        export_params.insert("include_admin".to_string(), "True".to_string());

        let export_result = list_api_keys(State(auth_manager.clone()), Query(export_params)).await;
        let (status, export_data) =
            extract_response_data::<serde_json::Value>(export_result).await?;
        assert_eq!(status, StatusCode::OK);
        let export_response = export_data.unwrap();
        let exported_keys: Vec<ApiKey> =
            serde_json::from_value(export_response["api_keys"].clone()).unwrap();
        assert_eq!(exported_keys.len(), 2);

        // Add an additional API key to the existing storage
        let additional_key = ApiKey {
            id: "additional-key".to_string(),
            role: Role::User,
            requests_today: 5,
            requests_this_minute: 1,
            last_request_minute: Some(chrono::Utc::now()),
            last_request_day: Some(chrono::Utc::now()),
        };
        auth_manager.storage.create(additional_key.clone()).await?;

        // Verify we now have 3 keys total using list handler with full details
        let mut params_before = HashMap::new();
        params_before.insert("include_admin".to_string(), "true".to_string());

        let export_before_import =
            list_api_keys(State(auth_manager.clone()), Query(params_before)).await;
        let (status, export_data) =
            extract_response_data::<serde_json::Value>(export_before_import).await?;
        assert_eq!(status, StatusCode::OK);
        let export_response = export_data.unwrap();
        let keys_before: Vec<ApiKey> =
            serde_json::from_value(export_response["api_keys"].clone()).unwrap();
        assert_eq!(keys_before.len(), 3);

        // Import the exported data (without clear_existing, i.e. merge mode)
        let import_data = ImportRequest {
            api_keys: exported_keys
                .into_iter()
                .map(|k| ImportApiKey {
                    id: k.id,
                    role: k.role,
                    requests_today: k.requests_today,
                    requests_this_minute: k.requests_this_minute,
                    last_request_minute: k.last_request_minute,
                    last_request_day: k.last_request_day,
                })
                .collect(),
            clear_existing: false, // Default merge mode
        };

        // Test import handler logic (we simulate the handler since Json extractor is hard to mock)
        let import_result = {
            // This simulates what the import_api_keys handler does
            let full_keys_for_handler: Vec<ApiKey> = import_data
                .api_keys
                .iter()
                .map(|import_key| ApiKey {
                    id: import_key.id.clone(),
                    role: import_key.role,
                    requests_today: import_key.requests_today,
                    requests_this_minute: import_key.requests_this_minute,
                    last_request_minute: import_key.last_request_minute,
                    last_request_day: import_key.last_request_day,
                })
                .collect();

            match auth_manager
                .storage
                .create_batch(full_keys_for_handler, import_data.clear_existing)
                .await
            {
                Ok(_) => Ok(StatusCode::OK),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        };

        // Verify import handler would return OK
        assert_eq!(import_result.unwrap(), StatusCode::OK);

        // Validate that the third key is preserved and the other two keys are replaced using list handler
        let mut params_after = HashMap::new();
        params_after.insert("include_admin".to_string(), "true".to_string());

        let export_after_import =
            list_api_keys(State(auth_manager.clone()), Query(params_after)).await;
        let (status, export_data) =
            extract_response_data::<serde_json::Value>(export_after_import).await?;
        assert_eq!(status, StatusCode::OK);
        let export_response = export_data.unwrap();
        let keys_after_import: Vec<ApiKey> =
            serde_json::from_value(export_response["api_keys"].clone()).unwrap();
        assert_eq!(keys_after_import.len(), 3);

        // The additional key should still exist
        let additional_preserved = keys_after_import
            .iter()
            .any(|k| k.id == "additional-key" && k.requests_today == 5);
        assert!(additional_preserved);

        // The original user key should be present (replaced/updated)
        let user_key_present = keys_after_import.iter().any(|k| k.id == user_key);
        assert!(user_key_present);

        // The admin key should be present
        let admin_key_present = keys_after_import
            .iter()
            .any(|k| k.id == "test-admin-key" && k.role == Role::Admin);
        assert!(admin_key_present);

        // Test clear_existing = true
        let clear_import_data = ImportRequest {
            api_keys: vec![ImportApiKey {
                id: "only-key".to_string(),
                role: Role::User,
                requests_today: 10,
                requests_this_minute: 2,
                last_request_minute: None,
                last_request_day: None,
            }],
            clear_existing: true,
        };

        // Test clear import handler logic
        let clear_import_result = {
            // This simulates what the import_api_keys handler does with clear_existing=true
            let full_keys_for_handler: Vec<ApiKey> = clear_import_data
                .api_keys
                .iter()
                .map(|import_key| ApiKey {
                    id: import_key.id.clone(),
                    role: import_key.role,
                    requests_today: import_key.requests_today,
                    requests_this_minute: import_key.requests_this_minute,
                    last_request_minute: import_key.last_request_minute,
                    last_request_day: import_key.last_request_day,
                })
                .collect();

            match auth_manager
                .storage
                .create_batch(full_keys_for_handler, clear_import_data.clear_existing)
                .await
            {
                Ok(_) => Ok(StatusCode::OK),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        };

        // Verify clear import handler would return OK
        assert_eq!(clear_import_result.unwrap(), StatusCode::OK);

        // Should only have 1 key now - verify using list handler with full details
        let mut params_final = HashMap::new();
        params_final.insert("include_admin".to_string(), "true".to_string());

        let export_after_clear =
            list_api_keys(State(auth_manager.clone()), Query(params_final)).await;
        let (status, export_data) =
            extract_response_data::<serde_json::Value>(export_after_clear).await?;
        assert_eq!(status, StatusCode::OK);
        let export_response = export_data.unwrap();
        let keys_after_clear: Vec<ApiKey> =
            serde_json::from_value(export_response["api_keys"].clone()).unwrap();
        assert_eq!(keys_after_clear.len(), 1);
        assert_eq!(keys_after_clear[0].id, "only-key");
        assert_eq!(keys_after_clear[0].requests_today, 10);

        Ok(())
    }
}
