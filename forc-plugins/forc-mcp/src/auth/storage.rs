use super::ApiKey;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage trait for API keys
#[async_trait]
pub trait ApiKeyStorage: Send + Sync {
    async fn create(&self, key: ApiKey) -> Result<()>;
    async fn get(&self, hash: &str) -> Result<Option<ApiKey>>;
    async fn update(&self, hash: &str, key: ApiKey) -> Result<()>;
    async fn delete(&self, hash: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<ApiKey>>;

    /// Create multiple API keys in batch with optional clearing
    async fn create_batch(&self, keys: Vec<ApiKey>, clear_existing: bool) -> Result<()>;
}

/// In-memory storage implementation
pub struct InMemoryStorage {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ApiKeyStorage for InMemoryStorage {
    async fn create(&self, key: ApiKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id.clone(), key);
        Ok(())
    }

    async fn get(&self, hash: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().find(|k| k.id == hash).cloned())
    }

    async fn update(&self, hash: &str, key: ApiKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(hash.to_string(), key);
        Ok(())
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(hash);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }

    async fn create_batch(&self, keys: Vec<ApiKey>, clear_existing: bool) -> Result<()> {
        let mut storage = self.keys.write().await;
        if clear_existing {
            storage.clear();
        }
        for key in keys {
            storage.insert(key.id.clone(), key);
        }
        Ok(())
    }
}

/// File-based storage implementation
pub struct FileStorage {
    file_path: String,
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl FileStorage {
    pub async fn new(file_path: &str) -> Result<Self> {
        let keys = if Path::new(file_path).exists() {
            let contents = tokio::fs::read_to_string(file_path).await?;
            let keys: Vec<ApiKey> = serde_json::from_str(&contents)?;
            keys.into_iter().map(|k| (k.id.clone(), k)).collect()
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path: file_path.to_string(),
            keys: Arc::new(RwLock::new(keys)),
        })
    }

    async fn save(&self) -> Result<()> {
        let keys = self.keys.read().await;
        let keys_vec: Vec<&ApiKey> = keys.values().collect();
        let json = serde_json::to_string_pretty(&keys_vec)?;
        tokio::fs::write(&self.file_path, json).await?;
        Ok(())
    }
}

#[async_trait]
impl ApiKeyStorage for FileStorage {
    async fn create(&self, key: ApiKey) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.insert(key.id.clone(), key);
        }
        self.save().await
    }

    async fn get(&self, hash: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().find(|k| k.id == hash).cloned())
    }

    async fn update(&self, hash: &str, key: ApiKey) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.insert(hash.to_string(), key);
        }
        self.save().await
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.remove(hash);
        }
        self.save().await
    }

    async fn list(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }

    async fn create_batch(&self, keys: Vec<ApiKey>, clear_existing: bool) -> Result<()> {
        {
            let mut storage = self.keys.write().await;
            if clear_existing {
                storage.clear();
            }
            for key in keys {
                storage.insert(key.id.clone(), key);
            }
        }
        self.save().await
    }
}
