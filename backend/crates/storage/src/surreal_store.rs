use mnemo_core::error::{MnemoError, MnemoResult};
use serde::de::DeserializeOwned;
use serde_json::Value;
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;
use tokio::sync::OnceCell;

type Db = Surreal<Any>;

pub struct SurrealStore {
    db: Db,
}

static GLOBAL: OnceCell<SurrealStore> = OnceCell::const_new();

impl SurrealStore {
    pub async fn connect_from_env() -> MnemoResult<Self> {
        let raw_endpoint = std::env::var("SURREALDB_RPC_URL")
            .or_else(|_| std::env::var("SURREALDB_URL"))
            .unwrap_or_else(|_| "ws://surrealdb:8000".into());
        let endpoint = normalize_ws_endpoint(raw_endpoint);
        let user = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "root".into());
        let pass = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".into());
        let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
        let db_name = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());

        tracing::info!("Surreal connect => url={} ns={} db={}", endpoint, ns, db_name);

        let db = any::connect(&endpoint).await.context("surreal connect via ws")?;
        db.signin(Root { username: &user, password: &pass }).await.context("surreal signin")?;
        db.use_ns(ns).use_db(db_name).await.context("surreal use_ns/use_db")?;

        Ok(Self { db })
    }

    pub async fn get() -> MnemoResult<&'static SurrealStore> {
        GLOBAL.get_or_try_init(|| async { Self::connect_from_env().await }).await
    }

    pub async fn exec(&self, sql: &str) -> MnemoResult<()> {
        let res = self.db.query(sql).await.context("surreal exec")?;
        res.check().context("surreal exec check")?;
        Ok(())
    }

    pub async fn select_all(&self, sql: &str) -> MnemoResult<Vec<Value>> {
        let mut res = self.db.query(sql).await.context("surreal query")?;
        let data: Vec<surrealdb::sql::Value> =
            res.take(0).context("surreal decode rows (surreal value)")?;
        Ok(data.into_iter().map(|v| v.into_json()).collect())
    }

    pub async fn select_count(&self, sql: &str) -> MnemoResult<i64> {
        let mut res = self.db.query(sql).await.context("surreal count query")?;
        let rows: Vec<Value> = res.take(0).context("surreal count decode")?;
        Ok(parse_count(rows.first()).unwrap_or(0))
    }

    pub async fn query_typed<T: DeserializeOwned>(&self, sql: &str) -> MnemoResult<Vec<T>> {
        let mut res = self.db.query(sql).await.context("surreal query typed")?;
        let rows: Vec<T> = res.take(0).context("surreal decode typed")?;
        Ok(rows)
    }

    pub async fn query_typed_bind<T, B>(&self, sql: &str, bindings: B) -> MnemoResult<Vec<T>>
    where
        T: DeserializeOwned,
        B: serde::Serialize + 'static,
    {
        let mut res =
            self.db.query(sql).bind(bindings).await.context("surreal query typed bind")?;
        let rows: Vec<T> = res.take(0).context("surreal decode typed bind")?;
        Ok(rows)
    }
}

fn normalize_ws_endpoint(raw: String) -> String {
    if raw.starts_with("http://") {
        raw.replacen("http://", "ws://", 1)
    } else if raw.starts_with("https://") {
        raw.replacen("https://", "wss://", 1)
    } else {
        raw
    }
}

trait ResultExt<T> {
    fn context(self, msg: &str) -> MnemoResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> MnemoResult<T> {
        self.map_err(|e| MnemoError::Message(format!("{msg}: {e}")))
    }
}

fn parse_count(v: Option<&Value>) -> Option<i64> {
    let v = v?;
    match v {
        Value::Number(n) => n.as_i64(),
        Value::Object(map) => {
            if let Some(n) = map.get("c").and_then(|x| x.as_i64()) {
                return Some(n);
            }
            if let Some(n) = map.get("count").and_then(|x| x.as_i64()) {
                return Some(n);
            }
            None
        }
        _ => None,
    }
}
