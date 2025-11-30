use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RagQueryRequest {
    pub session: Option<Uuid>,
    pub query: String,
}

#[derive(serde::Serialize)]
pub struct SessionResponse {
    pub session_id: Uuid,
    pub response: String,
}

pub async fn rag_query(Json(req): Json<RagQueryRequest>) -> Json<SessionResponse> {
    let session_id = req.session.unwrap_or_else(Uuid::new_v4);
    if req.session.is_none() {
        // create session record
        let pg_url = std::env::var("MNEMO_METADATA_PG")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mnemo".into());
        let store = mnemo_storage::metadata::postgres::PostgresMetadataStore::new(pg_url)
            .expect("pg store");
        let _ = store.create_session(session_id).await;
    }

    let query_text = req.query;
    let response = format!("rag_response_for: {}", query_text);

    // append history
    let pg_url = std::env::var("MNEMO_METADATA_PG")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mnemo".into());
    if let Ok(store) = mnemo_storage::metadata::postgres::PostgresMetadataStore::new(pg_url) {
        if let Ok(existing) = store.get_session_history(session_id).await {
            let mut history = existing.unwrap_or_else(|| serde_json::json!([]));
            if let Some(arr) = history.as_array_mut() {
                arr.push(serde_json::json!({ "query": query_text, "response": response }));
                if arr.len() > 10 {
                    let len = arr.len();
                    arr.drain(0..(len - 10));
                }
            }
            let _ = store.append_session_history(session_id, &history).await;
        }
    }

    Json(SessionResponse { session_id, response })
}
