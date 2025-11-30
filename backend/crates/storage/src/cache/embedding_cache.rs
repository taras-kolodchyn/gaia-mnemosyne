use super::redis::RedisCacheStore;

/// Placeholder embedding cache backed by Redis.
pub struct EmbeddingCache {
    pub store: RedisCacheStore,
}

impl EmbeddingCache {
    pub fn new(store: RedisCacheStore) -> Self {
        Self { store }
    }

    pub async fn get(&self, _key: &str) -> Option<Vec<f32>> {
        None
    }

    pub async fn set(&self, _key: &str, _value: Vec<f32>) {
        // no-op placeholder
    }
}
