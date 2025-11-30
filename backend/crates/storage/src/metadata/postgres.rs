use super::postgres_pool::PG_POOL;
use mnemo_core::error::{MnemoError, MnemoResult};
use mnemo_core::models::{chunk::Chunk, document::Document};
use once_cell::sync::OnceCell;
use sqlx::PgPool;

/// PostgreSQL-backed metadata store for fingerprints, files, chunks, and jobs.
#[derive(Clone)]
pub struct PostgresMetadataStore {
    pub pool: PgPool,
    init: OnceCell<()>,
}

impl PostgresMetadataStore {
    /// Build a store using a PostgreSQL connection string. Uses lazy connection to avoid
    /// blocking during construction; the first query will establish the pool.
    pub fn new(database_url: String) -> MnemoResult<Self> {
        // Initialize global pool once using provided URL.
        if std::env::var("DATABASE_URL").is_err() {
            // Safe in this context: only used to seed the global pool if unset.
            unsafe {
                std::env::set_var("DATABASE_URL", database_url);
            }
        }
        Ok(Self { pool: PG_POOL.clone(), init: OnceCell::new() })
    }

    fn namespace_key(namespace: &str) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        namespace.hash(&mut hasher);
        (hasher.finish() & 0x7FFF_FFFF_FFFF_FFFF) as i64
    }

    /// Try to acquire a Postgres advisory lock for a given namespace. Returns true if acquired.
    pub async fn try_advisory_lock(&self, namespace: &str) -> MnemoResult<bool> {
        let key = Self::namespace_key(namespace);
        let row: (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1) AS locked")
            .bind(key)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("advisory_lock failed: {e}")))?;
        Ok(row.0)
    }

    /// Release a previously taken advisory lock.
    pub async fn advisory_unlock(&self, namespace: &str) -> MnemoResult<()> {
        let key = Self::namespace_key(namespace);
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("advisory_unlock failed: {e}")))?;
        Ok(())
    }

    async fn ensure_schema(&self) -> MnemoResult<()> {
        if self.init.get().is_some() {
            return Ok(());
        }

        let ddl = [
            r#"CREATE TABLE IF NOT EXISTS fingerprints(
                path TEXT PRIMARY KEY,
                hash TEXT NOT NULL
            );"#,
            r#"CREATE TABLE IF NOT EXISTS files(
                id SERIAL PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                namespace TEXT,
                modified_at TIMESTAMPTZ,
                file_size BIGINT,
                file_type TEXT,
                metadata JSONB,
                created_at TIMESTAMPTZ DEFAULT now()
            );"#,
            r#"CREATE TABLE IF NOT EXISTS chunks(
                id SERIAL PRIMARY KEY,
                file_id INTEGER REFERENCES files(id) ON DELETE CASCADE,
                idx INTEGER NOT NULL,
                tags TEXT[],
                vector_id TEXT,
                created_at TIMESTAMPTZ DEFAULT now()
            );"#,
            r#"CREATE TABLE IF NOT EXISTS file_chunks(
                file_id INTEGER REFERENCES files(id) ON DELETE CASCADE,
                chunk_id INTEGER REFERENCES chunks(id) ON DELETE CASCADE,
                chunk_index INTEGER NOT NULL,
                PRIMARY KEY(file_id, chunk_id)
            );"#,
            r#"CREATE TABLE IF NOT EXISTS jobs(
                id UUID PRIMARY KEY,
                job_type TEXT,
                state TEXT,
                created_at TIMESTAMPTZ DEFAULT now(),
                updated_at TIMESTAMPTZ DEFAULT now()
            );"#,
            r#"CREATE TABLE IF NOT EXISTS file_dependencies(
                file_path TEXT NOT NULL,
                dep_path TEXT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT now(),
                PRIMARY KEY(file_path, dep_path)
            );"#,
            r#"CREATE TABLE IF NOT EXISTS ontology_rules(
                id SERIAL PRIMARY KEY,
                tag TEXT NOT NULL,
                patterns TEXT[] NOT NULL
            );"#,
            r#"CREATE TABLE IF NOT EXISTS rag_sessions(
                id UUID PRIMARY KEY,
                history JSONB,
                created_at TIMESTAMPTZ DEFAULT now()
            );"#,
            r#"CREATE TABLE IF NOT EXISTS semantic_clusters(
                doc_id TEXT NOT NULL,
                cluster_id INT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT now()
            );"#,
            r#"ALTER TABLE files ADD COLUMN IF NOT EXISTS metadata JSONB;"#,
        ];

        for stmt in ddl {
            sqlx::query(stmt)
                .execute(&self.pool)
                .await
                .map_err(|e| MnemoError::Message(format!("failed to init metadata schema: {e}")))?;
        }

        let _ = self.init.set(());
        Ok(())
    }

    pub async fn get_fingerprint(&self, path: &str) -> MnemoResult<Option<String>> {
        self.ensure_schema().await?;
        tracing::debug!("Metadata: lookup fingerprint for {}", path);
        let row: Option<(String,)> =
            sqlx::query_as("SELECT hash FROM fingerprints WHERE path = $1")
                .bind(path)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| MnemoError::Message(format!("fingerprint lookup failed: {e}")))?;
        Ok(row.map(|(hash,)| hash))
    }

    pub async fn set_fingerprint(&self, path: &str, hash: &str) -> MnemoResult<()> {
        self.ensure_schema().await?;
        tracing::debug!("Metadata: set fingerprint for {}", path);
        sqlx::query(
            "INSERT INTO fingerprints(path, hash) VALUES ($1, $2)
             ON CONFLICT(path) DO UPDATE SET hash = EXCLUDED.hash",
        )
        .bind(path)
        .bind(hash)
        .execute(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("set_fingerprint failed: {e}")))?;
        Ok(())
    }

    pub async fn insert_document_metadata(&self, doc: &Document) -> MnemoResult<()> {
        self.ensure_schema().await?;
        self.set_fingerprint(&doc.path, &doc.fingerprint).await?;
        sqlx::query(
            "INSERT INTO files(path, namespace, modified_at, file_size, file_type, metadata) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(path) DO UPDATE SET namespace = EXCLUDED.namespace,
                 modified_at = EXCLUDED.modified_at,
                 file_size = EXCLUDED.file_size,
                 file_type = EXCLUDED.file_type,
                 metadata = EXCLUDED.metadata",
        )
        .bind(&doc.path)
        .bind(&doc.namespace)
        .bind(doc.modified_at)
        .bind(doc.file_size)
        .bind(&doc.file_type)
        .bind(&doc.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("insert_document_metadata failed: {e}")))?;
        Ok(())
    }

    /// Insert or update a document minimal metadata (path, namespace, file_type). Returns document id.
    pub async fn insert_document(
        &self,
        path: &str,
        namespace: &str,
        file_type: &str,
    ) -> MnemoResult<i64> {
        self.ensure_schema().await?;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO files(path, namespace, file_type) VALUES ($1, $2, $3)
             ON CONFLICT(path) DO UPDATE SET namespace = EXCLUDED.namespace, file_type = EXCLUDED.file_type
             RETURNING id",
        )
        .bind(path)
        .bind(namespace)
        .bind(file_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("insert_document failed: {e}")))?;
        Ok(id)
    }

    pub async fn store_chunk_metadata(&self, doc: &Document, chunk: &Chunk) -> MnemoResult<()> {
        self.ensure_schema().await?;
        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files(path, namespace, modified_at, file_size, file_type, metadata) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(path) DO UPDATE SET namespace = EXCLUDED.namespace,
                 modified_at = EXCLUDED.modified_at,
                 file_size = EXCLUDED.file_size,
                 file_type = EXCLUDED.file_type,
                 metadata = EXCLUDED.metadata
             RETURNING id",
        )
        .bind(&doc.path)
        .bind(&doc.namespace)
        .bind(doc.modified_at)
        .bind(doc.file_size)
        .bind(&doc.file_type)
        .bind(&doc.metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("store_chunk_metadata file lookup failed: {e}")))?;

        let chunk_id: i64 = sqlx::query_scalar(
            "INSERT INTO chunks(file_id, idx, tags, vector_id) VALUES ($1, $2, $3, $4)
             ON CONFLICT (file_id, idx) DO UPDATE SET tags = EXCLUDED.tags, vector_id = EXCLUDED.vector_id
             RETURNING id",
        )
        .bind(file_id)
        .bind(chunk.chunk_index as i32)
        .bind(&chunk.tags)
        .bind(&chunk.vector_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("store_chunk_metadata failed: {e}")))?;

        // Maintain file -> chunk mapping
        let _ = sqlx::query(
            "INSERT INTO file_chunks(file_id, chunk_id, chunk_index) VALUES ($1, $2, $3)
             ON CONFLICT (file_id, chunk_id) DO UPDATE SET chunk_index = EXCLUDED.chunk_index",
        )
        .bind(file_id)
        .bind(chunk_id)
        .bind(chunk.chunk_index as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("file_chunks mapping failed: {e}")))?;
        Ok(())
    }

    /// Insert or update a chunk record linked to a document by id.
    pub async fn insert_chunk(
        &self,
        document_id: i64,
        text_hash: &str,
        chunk_index: usize,
    ) -> MnemoResult<i64> {
        self.ensure_schema().await?;
        let chunk_id: i64 = sqlx::query_scalar(
            "INSERT INTO chunks(file_id, idx, tags, vector_id) VALUES ($1, $2, '{}', $3)
             ON CONFLICT (file_id, idx) DO UPDATE SET vector_id = EXCLUDED.vector_id
             RETURNING id",
        )
        .bind(document_id)
        .bind(chunk_index as i32)
        .bind(text_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("insert_chunk failed: {e}")))?;

        let _ = sqlx::query(
            "INSERT INTO file_chunks(file_id, chunk_id, chunk_index) VALUES ($1, $2, $3)
             ON CONFLICT (file_id, chunk_id) DO UPDATE SET chunk_index = EXCLUDED.chunk_index",
        )
        .bind(document_id)
        .bind(chunk_id)
        .bind(chunk_index as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("file_chunks mapping failed: {e}")))?;

        Ok(chunk_id)
    }

    /// Record a file-level dependency. Returns Ok(true) if inserted, Ok(false) if it already existed.
    pub async fn record_dependency(&self, path: &str, dep: &str) -> MnemoResult<bool> {
        self.ensure_schema().await?;
        let rows = sqlx::query(
            "INSERT INTO file_dependencies(file_path, dep_path) VALUES ($1, $2)
             ON CONFLICT (file_path, dep_path) DO NOTHING",
        )
        .bind(path)
        .bind(dep)
        .execute(&self.pool)
        .await
        .map_err(|e| MnemoError::Message(format!("record_dependency failed: {e}")))?;
        Ok(rows.rows_affected() > 0)
    }

    /// Load ontology rules (tag + patterns).
    pub async fn load_ontology_rules(&self) -> MnemoResult<Vec<(String, Vec<String>)>> {
        self.ensure_schema().await?;
        let rows = sqlx::query("SELECT tag, patterns FROM ontology_rules")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("load_ontology_rules failed: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let tag: String = r.try_get("tag").unwrap_or_default();
                let patterns: Vec<String> = r.try_get("patterns").unwrap_or_default();
                (tag, patterns)
            })
            .collect())
    }

    pub async fn create_session(&self, id: uuid::Uuid) -> MnemoResult<()> {
        self.ensure_schema().await?;
        sqlx::query("INSERT INTO rag_sessions(id, history) VALUES ($1, $2)")
            .bind(id)
            .bind(serde_json::json!([]))
            .execute(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("create_session failed: {e}")))?;
        Ok(())
    }

    pub async fn append_session_history(
        &self,
        id: uuid::Uuid,
        history: &serde_json::Value,
    ) -> MnemoResult<()> {
        self.ensure_schema().await?;
        sqlx::query("UPDATE rag_sessions SET history = $2 WHERE id = $1")
            .bind(id)
            .bind(history)
            .execute(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("append_session_history failed: {e}")))?;
        Ok(())
    }

    pub async fn get_session_history(
        &self,
        id: uuid::Uuid,
    ) -> MnemoResult<Option<serde_json::Value>> {
        self.ensure_schema().await?;
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT history FROM rag_sessions WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| MnemoError::Message(format!("get_session_history failed: {e}")))?;
        Ok(row.map(|(h,)| h))
    }

    pub async fn save_clusters(&self, clusters: &[(String, i32)]) -> MnemoResult<()> {
        self.ensure_schema().await?;
        for (doc, cid) in clusters {
            sqlx::query("INSERT INTO semantic_clusters(doc_id, cluster_id) VALUES ($1, $2)")
                .bind(doc)
                .bind(cid)
                .execute(&self.pool)
                .await
                .map_err(|e| MnemoError::Message(format!("save_clusters failed: {e}")))?;
        }
        Ok(())
    }

    pub async fn list_clusters(&self) -> MnemoResult<Vec<(String, i32)>> {
        self.ensure_schema().await?;
        let rows = sqlx::query("SELECT doc_id, cluster_id FROM semantic_clusters")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MnemoError::Message(format!("list_clusters failed: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let doc: String = r.try_get("doc_id").unwrap_or_default();
                let cid: i32 = r.try_get("cluster_id").unwrap_or(0);
                (doc, cid)
            })
            .collect())
    }
}
use sqlx::Row;
