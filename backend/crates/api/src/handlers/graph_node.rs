use axum::{Json, extract::Path};
use mnemo_storage::surreal_store::SurrealStore;
use serde::Deserialize;
use serde::Serialize;
use surrealdb::sql::Thing;

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

#[derive(Debug, Deserialize)]
struct NodeQueryRow {
    id: Thing,
    path: String,
    namespace: String,
    kind: String,
    chunk_index: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct NeighborRow {
    neighbor: Thing,
}

fn parse_thing(id: &str) -> Option<Thing> {
    if let Ok(t) = id.parse::<Thing>() {
        return Some(t);
    }
    if !id.contains(':') {
        if let Ok(t) = format!("file:{id}").parse::<Thing>() {
            return Some(t);
        }
        if let Ok(t) = format!("chunk:{id}").parse::<Thing>() {
            return Some(t);
        }
    }
    None
}

fn format_label(row: &NodeQueryRow) -> String {
    if row.kind == "chunk" {
        if let Some(idx) = row.chunk_index {
            return format!("{}#{}", row.path, idx);
        }
    }
    row.path.clone()
}

pub async fn graph_node(Path(id): Path<String>) -> Json<GraphNodeDetail> {
    let node_thing = match parse_thing(&id) {
        Some(t) => t,
        None => {
            return Json(GraphNodeDetail {
                id: id.clone(),
                label: id.clone(),
                node_type: "unknown".into(),
                neighbors: Vec::new(),
            });
        }
    };

    let store = match SurrealStore::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal init failed: {err}");
            return Json(GraphNodeDetail {
                id: id.clone(),
                label: id.clone(),
                node_type: "unknown".into(),
                neighbors: Vec::new(),
            });
        }
    };

    let node_sql = r#"
        SELECT id, path, namespace, 'file' AS kind, null AS chunk_index FROM file WHERE id = $id
        UNION ALL
        SELECT id, path, namespace, 'chunk' AS kind, chunk_index FROM chunk WHERE id = $id
    "#;

    let node_rows: Vec<NodeQueryRow> =
        store.query_typed_bind(node_sql, ("id", node_thing.clone())).await.unwrap_or_default();
    tracing::info!("Graph node lookup rows={}", node_rows.len());

    let mut node_type = "unknown".to_string();
    let mut label = id.clone();
    if let Some(row) = node_rows.get(0) {
        node_type = row.kind.clone();
        label = format_label(row);
    }

    let neighbor_sql = r#"
        SELECT out AS neighbor FROM contains WHERE in = $id
        UNION ALL
        SELECT in AS neighbor FROM contains WHERE out = $id
    "#;
    let neighbor_rows: Vec<NeighborRow> =
        store.query_typed_bind(neighbor_sql, ("id", node_thing.clone())).await.unwrap_or_default();
    tracing::info!("Graph node neighbors raw rows={}", neighbor_rows.len());

    let mut neighbors = Vec::new();
    for row in neighbor_rows {
        let other = row.neighbor;
        let info_rows: Vec<NodeQueryRow> =
            store.query_typed_bind(node_sql, ("id", other.clone())).await.unwrap_or_default();
        let (mut n_label, mut n_type) = (other.to_string(), "unknown".to_string());
        if let Some(nr) = info_rows.get(0) {
            n_type = nr.kind.clone();
            n_label = format_label(nr);
        }
        neighbors.push(Neighbor { id: other.to_string(), label: n_label, node_type: n_type });
    }

    Json(GraphNodeDetail { id: id.clone(), label, node_type, neighbors })
}
