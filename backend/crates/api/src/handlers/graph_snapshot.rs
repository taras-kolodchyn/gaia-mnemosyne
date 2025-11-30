use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Serialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub data: GraphNodeData,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Serialize, Clone)]
pub struct GraphNodeData {
    pub label: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct GraphSnapshot {
    pub total_nodes: i64,
    pub returned: usize,
    pub limit: usize,
    pub offset: usize,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Deserialize)]
pub struct SnapshotParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

async fn fetch_sql(
    client: &reqwest::Client,
    base_url: &str,
    sql: &str,
    ns: &str,
    db: &str,
    user: &str,
    pass: &str,
) -> Vec<serde_json::Value> {
    tracing::debug!("Surreal SQL query: {}", sql);
    let res = client
        .post(format!("{}/sql", base_url))
        .header("NS", ns)
        .header("DB", db)
        .header("Content-Type", "text/plain")
        .header("Accept", "application/json")
        .basic_auth(user, Some(pass))
        .body(sql.to_string())
        .send()
        .await;
    match res {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::debug!(
                "Surreal response status={} body_len={} body_snip={}",
                status,
                text.len(),
                text.chars().take(2000).collect::<String>()
            );
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => {
                    if let Some(arr) = json.as_array() {
                        if let Some(obj) = arr.get(0).and_then(|v| v.as_object()) {
                            if let Some(rows) = obj.get("result").and_then(|r| r.as_array()) {
                                tracing::debug!("Surreal parsed rows={}", rows.len());
                                return rows.clone();
                            }
                        }
                    }
                    tracing::warn!("Surreal response missing result array");
                }
                Err(err) => {
                    tracing::error!("Surreal JSON parse failed: {err} body={}", text);
                }
            }
        }
        Err(err) => tracing::error!("Surreal request failed: {err}"),
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

pub async fn graph_snapshot(Query(params): Query<SnapshotParams>) -> Json<GraphSnapshot> {
    let limit = params.limit.unwrap_or(500);
    let offset = params.offset.unwrap_or(0);

    let surreal_url =
        std::env::var("SURREALDB_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let client = reqwest::Client::new();
    let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
    let db = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());
    let user = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "root".into());
    let pass = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".into());

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    let label_from = |v: &serde_json::Value| {
        v.get("path")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "node".into())
    };

    let file_rows = fetch_sql(
        &client,
        &surreal_url,
        "SELECT id, path, namespace FROM file;",
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    tracing::info!(
        "Graph snapshot: fetched {} file rows (sample={:?})",
        file_rows.len(),
        file_rows.iter().take(3).collect::<Vec<_>>()
    );
    for v in file_rows {
        if let Some(id) = v.get("id").and_then(extract_id) {
            nodes.push(GraphNode {
                id: id.to_string(),
                data: GraphNodeData { label: label_from(&v) },
                node_type: "file".into(),
            });
        }
    }

    let chunk_rows = fetch_sql(
        &client,
        &surreal_url,
        "SELECT id, path, namespace, chunk_index FROM chunk;",
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    tracing::info!("Graph snapshot: fetched {} chunk rows", chunk_rows.len());
    for v in chunk_rows {
        if let Some(id) = v.get("id").and_then(extract_id) {
            let label = v
                .get("path")
                .and_then(|p| p.as_str())
                .map(|p| {
                    format!("{}#{}", p, v.get("chunk_index").and_then(|i| i.as_i64()).unwrap_or(0))
                })
                .unwrap_or_else(|| label_from(&v));
            nodes.push(GraphNode {
                id: id.to_string(),
                data: GraphNodeData { label },
                node_type: "chunk".into(),
            });
        }
    }

    let edge_rows =
        fetch_sql(&client, &surreal_url, "SELECT in, out FROM contains;", &ns, &db, &user, &pass)
            .await;
    tracing::info!(
        "Graph snapshot: fetched {} edge rows (sample={:?})",
        edge_rows.len(),
        edge_rows.iter().take(3).collect::<Vec<_>>()
    );
    for v in edge_rows {
        if let (Some(src), Some(dst)) =
            (v.get("in").and_then(extract_id), v.get("out").and_then(extract_id))
        {
            let source = src.to_string();
            let target = dst.to_string();
            let id = format!("{}->{}", source, target);
            edges.push(GraphEdge { id, source, target });
        }
    }

    // Apply pagination to nodes and filter edges to returned nodes.
    let start = offset.min(nodes.len());
    let end = (offset + limit).min(nodes.len());
    let paged_nodes = nodes[start..end].to_vec();
    let node_ids: HashSet<String> = paged_nodes.iter().map(|n| n.id.clone()).collect();
    let filtered_edges: Vec<GraphEdge> = edges
        .into_iter()
        .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
        .collect();

    tracing::info!(
        "Graph snapshot response: nodes={}, edges={}, returned={}",
        nodes.len(),
        filtered_edges.len(),
        paged_nodes.len()
    );

    Json(GraphSnapshot {
        total_nodes: nodes.len() as i64,
        returned: paged_nodes.len(),
        limit,
        offset,
        nodes: paged_nodes,
        edges: filtered_edges,
    })
}

pub async fn graph_debug() -> Json<serde_json::Value> {
    let surreal_url =
        std::env::var("SURREALDB_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let client = reqwest::Client::new();
    let ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "mnemo".into());
    let db = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "mnemo".into());
    let user = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "root".into());
    let pass = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".into());

    let files = fetch_sql(
        &client,
        &surreal_url,
        "SELECT id, path, namespace FROM file;",
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    let chunks = fetch_sql(
        &client,
        &surreal_url,
        "SELECT id, path, namespace, chunk_index FROM chunk;",
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;
    let edges = fetch_sql(
        &client,
        &surreal_url,
        "SELECT in, out FROM contains;",
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;

    tracing::info!(
        "Graph debug: files={}, chunks={}, edges={}",
        files.len(),
        chunks.len(),
        edges.len()
    );

    Json(serde_json::json!({
        "files": files,
        "chunks": chunks,
        "edges": edges
    }))
}
