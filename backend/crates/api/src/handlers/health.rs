use serde::Serialize;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_postgres::{Config as PgConfig, NoTls};

#[derive(Serialize)]
pub struct HealthResponse {
    pub api: String,
    pub qdrant: String,
    pub surrealdb: String,
    pub redis: String,
    pub postgres: String,
}

fn up_or_down(ok: bool) -> String {
    if ok { "UP".into() } else { "DOWN".into() }
}

async fn check_qdrant(client: &reqwest::Client) -> String {
    let res = client
        .get("http://qdrant:6333/readyz")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    up_or_down(res)
}

async fn check_surrealdb() -> String {
    let res = TcpStream::connect("surrealdb:8000").await.is_ok();
    up_or_down(res)
}

async fn check_redis() -> String {
    let client = match redis::Client::open("redis://redis:6379/") {
        Ok(c) => c,
        Err(_) => return "DOWN".into(),
    };

    let res = async {
        let mut conn = client.get_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok::<_, redis::RedisError>(())
    }
    .await
    .is_ok();

    up_or_down(res)
}

async fn check_postgres() -> String {
    let db_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("MNEMO_METADATA_PG"))
        .unwrap_or_else(|_| "postgres://mnemo:mnemo@localhost:5432/mnemo".into());

    let res = if let Ok(cfg) = PgConfig::from_str(&db_url) {
        cfg.connect(NoTls).await.is_ok()
    } else {
        false
    };

    up_or_down(res)
}

pub async fn health() -> axum::Json<HealthResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let (qdrant, surrealdb, redis, postgres) =
        tokio::join!(check_qdrant(&client), check_surrealdb(), check_redis(), check_postgres());

    axum::Json(HealthResponse { api: "UP".into(), qdrant, surrealdb, redis, postgres })
}
