use axum::{extract::Query, Json};
use mnemo_storage::surreal_rpc_client::SurrealRpcClient;
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

async fn fetch_rows(client: &SurrealRpcClient, sql: &str) -> Vec<serde_json::Value> {
    tracing::debug!("Surreal RPC query: {}", sql);
    match client.query(sql).await {
        Ok(rows) => {
            tracing::debug!("Surreal RPC rows={}", rows.len());
            rows
        }
        Err(err) => {
            tracing::error!("Surreal RPC query failed: {err}");
            Vec::new()
        }
    }
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

    let client = match SurrealRpcClient::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal RPC init failed: {err}");
            return Json(GraphSnapshot {
                total_nodes: 0,
                returned: 0,
                limit,
                offset,
                nodes: Vec::new(),
                edges: Vec::new(),
            });
        }
    };

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    let label_from = |v: &serde_json::Value| {
        v.get("path")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "node".into())
    };

    let file_rows = fetch_rows(client, "SELECT id, path, namespace FROM file;").await;
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

    let chunk_rows =
        fetch_rows(client, "SELECT id, path, namespace, chunk_index FROM chunk;").await;
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

    let edge_rows = fetch_rows(client, "SELECT in, out FROM contains;").await;
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
    let client = match SurrealRpcClient::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal RPC init failed: {err}");
            return Json(serde_json::json!({
                "files": [],
                "chunks": [],
                "edges": [],
                "error": err.to_string()
            }));
        }
    };

    let files = fetch_rows(client, "SELECT id, path, namespace FROM file;").await;
    let chunks =
        fetch_rows(client, "SELECT id, path, namespace, chunk_index FROM chunk;").await;
    let edges = fetch_rows(client, "SELECT in, out FROM contains;").await;

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
