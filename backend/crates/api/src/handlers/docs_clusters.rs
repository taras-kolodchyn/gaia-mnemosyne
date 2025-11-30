use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ClusterEntry {
    pub doc_id: String,
    pub cluster_id: i32,
}

#[derive(Serialize)]
pub struct ClusterResponse {
    pub clusters: Vec<ClusterEntry>,
}

pub async fn docs_clusters() -> Json<ClusterResponse> {
    let pg_url = std::env::var("MNEMO_METADATA_PG")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mnemo".into());

    let clusters =
        if let Ok(store) = mnemo_storage::metadata::postgres::PostgresMetadataStore::new(pg_url) {
            store
                .list_clusters()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|(doc_id, cluster_id)| ClusterEntry { doc_id, cluster_id })
                .collect()
        } else {
            Vec::new()
        };

    Json(ClusterResponse { clusters })
}
