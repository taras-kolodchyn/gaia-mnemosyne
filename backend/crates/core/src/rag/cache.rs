use redis::AsyncCommands;
use serde_json;
use sha1::Digest;

use crate::models::rag_context::RAGContext;

fn cache_key(query: &str) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(query.as_bytes());
    let digest = hasher.finalize();
    format!("rag:{:x}", digest)
}

pub async fn get_cached(query: &str) -> Option<RAGContext> {
    let url = std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into());
    let client = redis::Client::open(url).ok()?;
    let mut conn = client.get_async_connection().await.ok()?;
    let key = cache_key(query);
    let val: Option<String> = conn.get(key).await.ok()?;
    val.and_then(|v| serde_json::from_str(&v).ok())
}

pub async fn set_cached(query: &str, ctx: &RAGContext) {
    let url = std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into());
    if let Ok(client) = redis::Client::open(url) {
        if let Ok(mut conn) = client.get_async_connection().await {
            if let Ok(val) = serde_json::to_string(ctx) {
                let _: redis::RedisResult<()> = conn.set_ex(cache_key(query), val, 600).await;
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChunkCacheEntry {
    pub text: String,
    pub tags: Vec<String>,
    pub file_path: String,
}

fn chunk_key(id: &str) -> String {
    format!("chunk:{id}")
}

pub async fn get_chunk_cached(id: &str) -> Option<ChunkCacheEntry> {
    let url = std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into());
    let client = redis::Client::open(url).ok()?;
    let mut conn = client.get_async_connection().await.ok()?;
    let key = chunk_key(id);
    let val: Option<String> = conn.get(key).await.ok()?;
    val.and_then(|v| serde_json::from_str(&v).ok())
}

pub async fn set_chunk_cached(id: &str, entry: &ChunkCacheEntry) {
    let url = std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into());
    if let Ok(client) = redis::Client::open(url) {
        if let Ok(mut conn) = client.get_async_connection().await {
            if let Ok(val) = serde_json::to_string(entry) {
                // TTL 1 hour
                let _: redis::RedisResult<()> = conn.set_ex(chunk_key(id), val, 3600).await;
            }
        }
    }
}
