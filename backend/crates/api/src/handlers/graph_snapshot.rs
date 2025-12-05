use axum::{Json, extract::Query};
use mnemo_storage::surreal_store::SurrealStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use surrealdb::sql::{Id, Thing};

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

#[derive(Debug, Deserialize, Serialize)]
struct FileRow {
    id: Thing,
    namespace: String,
    path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChunkRow {
    id: Thing,
    namespace: String,
    path: String,
    chunk_index: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct EdgeRow {
    #[serde(rename = "in")]
    in_node: Thing,
    out: Thing,
}

fn normalize_thing(thing: Thing, expected_tb: &str) -> Thing {
    let mut id_str = match thing.id {
        Id::String(s) => s,
        Id::Number(n) => n.to_string(),
        Id::Uuid(u) => u.to_string(),
        other => other.to_string(),
    };
    if let Some((_, rest)) = id_str.split_once(':') {
        id_str = rest.to_string();
    }
    Thing::from((expected_tb, id_str.as_str()))
}

fn thing_to_string(thing: &Thing) -> String {
    thing.to_string()
}

pub async fn graph_snapshot(Query(params): Query<SnapshotParams>) -> Json<GraphSnapshot> {
    let limit = params.limit.unwrap_or(500);
    let offset = params.offset.unwrap_or(0);

    let store = match SurrealStore::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal init failed: {err}");
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

    let file_rows: Vec<FileRow> =
        match store.query_typed("SELECT id, path, namespace FROM file;").await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!("Surreal file query failed: {err}");
                Vec::new()
            }
        };
    tracing::info!(
        "Graph snapshot: fetched {} file rows (sample={:?})",
        file_rows.len(),
        file_rows.iter().take(3).collect::<Vec<_>>()
    );
    let mut node_ids: HashSet<String> = HashSet::new();
    for mut row in file_rows {
        row.id = normalize_thing(row.id, "file");
        let id = thing_to_string(&row.id);
        if !node_ids.insert(id.clone()) {
            continue;
        }
        nodes.push(GraphNode {
            id: id.clone(),
            data: GraphNodeData { label: row.path },
            node_type: "file".into(),
        });
    }

    let chunk_rows: Vec<ChunkRow> =
        match store.query_typed("SELECT id, path, namespace, chunk_index FROM chunk;").await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!("Surreal chunk query failed: {err}");
                Vec::new()
            }
        };
    tracing::info!("Graph snapshot: fetched {} chunk rows", chunk_rows.len());
    for mut row in chunk_rows {
        row.id = normalize_thing(row.id, "chunk");
        let id = thing_to_string(&row.id);
        if !node_ids.insert(id.clone()) {
            continue;
        }
        let label = format!("{}#{}", row.path, row.chunk_index);
        nodes.push(GraphNode {
            id: id.clone(),
            data: GraphNodeData { label },
            node_type: "chunk".into(),
        });
    }

    let edge_rows: Vec<EdgeRow> = match store.query_typed("SELECT in, out FROM contains;").await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!("Surreal edge query failed: {err}");
            Vec::new()
        }
    };
    tracing::info!(
        "Graph snapshot: fetched {} edge rows (sample={:?})",
        edge_rows.len(),
        edge_rows.iter().take(3).collect::<Vec<_>>()
    );
    let mut edge_ids: HashSet<String> = HashSet::new();
    for mut row in edge_rows {
        row.in_node = normalize_thing(row.in_node, "file");
        row.out = normalize_thing(row.out, "chunk");
        let source = thing_to_string(&row.in_node);
        let target = thing_to_string(&row.out);
        let id = format!("{}->{}", source, target);
        if edge_ids.insert(id.clone()) {
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
    let store = match SurrealStore::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal init failed: {err}");
            return Json(serde_json::json!({
                "files": [],
                "chunks": [],
                "edges": [],
                "error": err.to_string()
            }));
        }
    };

    let files: Vec<FileRow> =
        store.query_typed("SELECT id, path, namespace FROM file;").await.unwrap_or_default();
    let chunks: Vec<ChunkRow> = store
        .query_typed("SELECT id, path, namespace, chunk_index FROM chunk;")
        .await
        .unwrap_or_default();
    let edges: Vec<EdgeRow> =
        store.query_typed("SELECT in, out FROM contains;").await.unwrap_or_default();

    tracing::info!(
        "Graph debug: files={}, chunks={}, edges={}",
        files.len(),
        chunks.len(),
        edges.len()
    );

    let files_json: Vec<Value> = files
        .into_iter()
        .map(|mut f| {
            f.id = normalize_thing(f.id, "file");
            serde_json::json!({
                "id": thing_to_string(&f.id),
                "namespace": f.namespace,
                "path": f.path
            })
        })
        .collect();
    let chunks_json: Vec<Value> = chunks
        .into_iter()
        .map(|mut c| {
            c.id = normalize_thing(c.id, "chunk");
            serde_json::json!({
                "id": thing_to_string(&c.id),
                "namespace": c.namespace,
                "path": c.path,
                "chunk_index": c.chunk_index
            })
        })
        .collect();
    let edges_json: Vec<Value> = edges
        .into_iter()
        .map(|mut e| {
            e.in_node = normalize_thing(e.in_node, "file");
            e.out = normalize_thing(e.out, "chunk");
            serde_json::json!({
                "in": thing_to_string(&e.in_node),
                "out": thing_to_string(&e.out)
            })
        })
        .collect();

    Json(serde_json::json!({
        "files": files_json,
        "chunks": chunks_json,
        "edges": edges_json
    }))
}
