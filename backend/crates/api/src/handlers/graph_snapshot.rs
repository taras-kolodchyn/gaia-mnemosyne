use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
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
    if let Ok(resp) = res {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(arr) = json.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
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
    tracing::info!("Graph snapshot: fetched {} file rows", file_rows.len());
    for v in file_rows {
        if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
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
        if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
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
    tracing::info!("Graph snapshot: fetched {} edge rows", edge_rows.len());
    for v in edge_rows {
        if let (Some(src), Some(dst)) =
            (v.get("in").and_then(|x| x.as_str()), v.get("out").and_then(|x| x.as_str()))
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
