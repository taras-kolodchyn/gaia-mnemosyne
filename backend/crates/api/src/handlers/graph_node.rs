use axum::{Json, extract::Path};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct Neighbor {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Serialize)]
pub struct GraphNodeDetail {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub neighbors: Vec<Neighbor>,
}

async fn fetch_sql(
    client: &reqwest::Client,
    sql: &str,
    base_url: &str,
    ns: &str,
    db: &str,
    user: &str,
    pass: &str,
) -> Vec<serde_json::Value> {
    tracing::debug!("Surreal SQL query (node): {}", sql);
    if let Ok(resp) = client
        .post(format!("{}/sql", base_url))
        .header("NS", ns)
        .header("DB", db)
        .header("Content-Type", "text/plain")
        .header("Accept", "application/json")
        .basic_auth(user, Some(pass))
        .body(sql.to_string())
        .send()
        .await
    {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::debug!(
            "Surreal node response status={} body_len={} body_snip={}",
            status,
            text.len(),
            text.chars().take(2000).collect::<String>()
        );
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(val) => {
                // SurrealDB 2.x wraps result in [{result:[...]}]
                if let Some(arr) = val.as_array() {
                    if let Some(obj) = arr.get(0).and_then(|v| v.as_object()) {
                        if let Some(rows) = obj.get("result").and_then(|r| r.as_array()) {
                            tracing::debug!("Surreal node parsed rows={}", rows.len());
                            return rows.clone();
                        }
                    }
                }
                tracing::warn!("Surreal node response missing result array");
            }
            Err(err) => {
                tracing::error!("Surreal node JSON parse failed: {err} body={}", text);
            }
        }
    }
    Vec::new()
}

fn extract_id(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(obj) = v.as_object() {
        if let (Some(tb), Some(id)) = (obj.get("tb").and_then(|x| x.as_str()), obj.get("id").and_then(|x| x.as_str())) {
            return Some(format!("{}:{}", tb, id));
        }
    }
    None
}

pub async fn graph_node(Path(id): Path<String>) -> Json<GraphNodeDetail> {
    let client = reqwest::Client::new();
    let surreal_url =
        std::env::var("SURREALDB_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
    let db = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());
    let user = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "root".into());
    let pass = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".into());

    let node_rows = fetch_sql(
        &client,
        &format!(
            "SELECT id, path, namespace, 'file' as kind FROM file WHERE id = '{id}' UNION ALL SELECT id, path, namespace, chunk_index, 'chunk' as kind FROM chunk WHERE id = '{id}';"
        ),
        &surreal_url,
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    tracing::info!("Graph node lookup rows={}", node_rows.len());

    let mut node_type = "unknown".to_string();
    let mut label = id.clone();
    if let Some(row) = node_rows.get(0) {
        if let Some(k) = row.get("kind").and_then(|v| v.as_str()) {
            node_type = k.to_string();
        }
        if let Some(p) = row.get("path").and_then(|v| v.as_str()) {
            if let Some(idx) = row.get("chunk_index").and_then(|c| c.as_i64()) {
                label = format!("{}#{}", p, idx);
            } else {
                label = p.to_string();
            }
        }
    }

    let neighbor_rows = fetch_sql(
        &client,
        &format!("SELECT in, out FROM contains WHERE in = '{id}' OR out = '{id}';"),
        &surreal_url,
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    tracing::info!("Graph node neighbors raw rows={}", neighbor_rows.len());

    let mut neighbors = Vec::new();
    for row in neighbor_rows {
        let in_id = row.get("in").and_then(extract_id);
        let out_id = row.get("out").and_then(extract_id);
        if let (Some(in_id), Some(out_id)) = (in_id, out_id) {
            let other = if in_id == id { out_id } else { in_id };
            let neighbor_rows = fetch_sql(
                &client,
                &format!(
                    "SELECT id, path, namespace, 'file' as kind FROM file WHERE id = '{other}' UNION ALL SELECT id, path, namespace, chunk_index, 'chunk' as kind FROM chunk WHERE id = '{other}';"
                ),
                &surreal_url,
                &ns,
                &db,
                &user,
                &pass,
            )
            .await;
            let (mut n_label, mut n_type) = (other.to_string(), "unknown".to_string());
            if let Some(nr) = neighbor_rows.get(0) {
                if let Some(k) = nr.get("kind").and_then(|v| v.as_str()) {
                    n_type = k.to_string();
                }
                if let Some(p) = nr.get("path").and_then(|v| v.as_str()) {
                    if let Some(idx) = nr.get("chunk_index").and_then(|c| c.as_i64()) {
                        n_label = format!("{}#{}", p, idx);
                    } else {
                        n_label = p.to_string();
                    }
                }
            }

            neighbors.push(Neighbor { id: other.to_string(), label: n_label, node_type: n_type });
        }
    }

    Json(GraphNodeDetail { id: id.clone(), label, node_type, neighbors })
}
