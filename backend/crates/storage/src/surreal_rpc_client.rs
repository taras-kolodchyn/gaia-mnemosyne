use serde_json::Value;
use tokio::sync::OnceCell;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Value as SurrealValue;
use surrealdb::Surreal;

type RpcDb = Surreal<Any>;

pub struct SurrealRpcClient {
    db: RpcDb,
}

static GLOBAL: OnceCell<SurrealRpcClient> = OnceCell::const_new();

impl SurrealRpcClient {
    async fn create() -> mnemo_core::error::MnemoResult<Self> {
        let url =
            std::env::var("SURREALDB_RPC_URL").unwrap_or_else(|_| "ws://surrealdb:8000".into());
        let user = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "root".into());
        let pass = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".into());
        let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
        let db_name = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());

        let db = any::connect(url).await.context("surreal connect via ws")?;
        db.signin(Root { username: &user, password: &pass })
            .await
            .context("surreal signin")?;
        db.use_ns(ns).use_db(db_name).await.context("surreal use_ns/use_db")?;

        Ok(Self { db })
    }

    pub async fn get() -> mnemo_core::error::MnemoResult<&'static SurrealRpcClient> {
        GLOBAL.get_or_try_init(|| async { Self::create().await }).await
    }

    pub async fn query(&self, sql: &str) -> mnemo_core::error::MnemoResult<Vec<Value>> {
        tracing::debug!("Surreal RPC SQL => {}", sql);
        let mut resp = self.db.query(sql).await.context("surreal query")?;
        let mut out: Vec<Value> = Vec::new();
        let num = resp.num_statements();
        for i in 0..num {
            // Try to read a list of values first (SELECT), otherwise fall back to a single value (INSERT/UPSERT).
            match resp.take::<Vec<SurrealValue>>(i) {
                Ok(values) => {
                    for v in values {
                        out.push(v.into_json());
                    }
                }
                Err(e) => {
                    tracing::trace!("Surreal RPC take list failed (idx {i}): {e}");
                    if let Ok(single) = resp.take::<SurrealValue>(i) {
                        out.push(single.into_json());
                    }
                }
            }
        }
        tracing::debug!("Surreal RPC RESULTS => {} rows", out.len());
        Ok(out)
    }

    pub async fn execute(&self, sql: &str) -> mnemo_core::error::MnemoResult<()> {
        let _ = self.db.query(sql).await.context("surreal exec")?;
        Ok(())
    }
}

trait ResultExt<T> {
    fn context(self, msg: &str) -> mnemo_core::error::MnemoResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> mnemo_core::error::MnemoResult<T> {
        self.map_err(|e| mnemo_core::error::MnemoError::Message(format!("{msg}: {e}")))
    }
}
