/// Placeholder Redis cache store adapter.
pub struct RedisCacheStore {
    pub url: String,
}

impl RedisCacheStore {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
