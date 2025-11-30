use once_cell::sync::Lazy;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Global Postgres connection pool for storage metadata.
pub static PG_POOL: Lazy<Pool<Postgres>> = Lazy::new(|| {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new().max_connections(5).connect_lazy(&url).expect("failed to init pg pool")
});
