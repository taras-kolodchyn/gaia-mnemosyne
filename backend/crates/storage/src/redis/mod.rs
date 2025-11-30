pub fn redis_url() -> String {
    std::env::var("MNEMO_REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into())
}

use once_cell::sync::OnceCell;
use redis::AsyncCommands;
use tokio::time::{Duration, sleep};

static REDIS_CLIENT: OnceCell<redis::Client> = OnceCell::new();

fn get_client() -> Option<redis::Client> {
    REDIS_CLIENT
        .get_or_init(|| {
            redis::Client::open(redis_url()).unwrap_or_else(|_| {
                redis::Client::open("redis://localhost:6379").expect("fallback redis client")
            })
        })
        .clone()
        .into()
}

pub async fn get_conn() -> Option<redis::aio::Connection> {
    let client = get_client()?;
    // Retry a few times in case the pooled connection is dead.
    let mut attempts = 0;
    loop {
        match client.get_async_connection().await {
            Ok(conn) => return Some(conn),
            Err(_) if attempts < 3 => {
                attempts += 1;
                sleep(Duration::from_millis(150 * attempts)).await;
                continue;
            }
            Err(_) => return None,
        }
    }
}

pub async fn cache_set_json(key: &str, value: &serde_json::Value, ttl_secs: usize) -> bool {
    if let Some(mut conn) = get_conn().await {
        if let Ok(s) = serde_json::to_string(value) {
            let _: redis::RedisResult<()> = conn.set_ex(key, s, ttl_secs as u64).await;
            return true;
        }
    }
    false
}

pub async fn cache_get_json(key: &str) -> Option<serde_json::Value> {
    if let Some(mut conn) = get_conn().await {
        let val: Option<String> = conn.get(key).await.ok()?;
        if let Some(v) = val {
            return serde_json::from_str(&v).ok();
        }
    }
    None
}
