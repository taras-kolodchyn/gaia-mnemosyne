use axum::{Json, extract::Path};
use mnemo_storage::surreal_store::SurrealStore;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use surrealdb::sql::{Id, Thing};

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

fn normalize_thing(mut thing: Thing, expected_tb: &str) -> Thing {
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

fn thing_to_string(t: &Thing) -> String {
    t.to_string()
}

fn infer_expected_tb(id_str: &str) -> &str {
    if id_str.starts_with("chunk:") {
        "chunk"
    } else if id_str.starts_with("file:") {
        "file"
    } else {
        "file"
    }
}

fn parse_thing(id: &str) -> Thing {
    let expected = infer_expected_tb(id);
    let thing = Thing::from((expected, id));
    normalize_thing(thing, expected)
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
    let node_thing = parse_thing(&id);

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
    let mut response_id = id.clone();
    if let Some(row) = node_rows.get(0) {
        let expected = if row.kind == "chunk" { "chunk" } else { "file" };
        let norm_id = normalize_thing(row.id.clone(), expected);
        node_type = row.kind.clone();
        label = format_label(row);
        let norm_id_str = thing_to_string(&norm_id);
        response_id = norm_id_str.clone();
        if norm_id_str != id {
            tracing::debug!("Normalized node id {} -> {}", id, norm_id_str);
        }
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
    let mut neighbor_seen: HashSet<String> = HashSet::new();
    for row in neighbor_rows {
        let mut other = row.neighbor;
        let other_str = other.to_string();
        let expected_neighbor_tb = match node_type.as_str() {
            "file" => "chunk",
            "chunk" => "file",
            _ => infer_expected_tb(&other_str),
        };
        other = normalize_thing(other, expected_neighbor_tb);
        let other_id_str = thing_to_string(&other);
        if !neighbor_seen.insert(other_id_str.clone()) {
            continue;
        }
        let info_rows: Vec<NodeQueryRow> =
            store.query_typed_bind(node_sql, ("id", other.clone())).await.unwrap_or_default();
        let (mut n_label, mut n_type) = (other_id_str.clone(), "unknown".to_string());
        if let Some(nr) = info_rows.get(0) {
            let expected_tb = if nr.kind == "chunk" { "chunk" } else { "file" };
            let normalized = normalize_thing(nr.id.clone(), expected_tb);
            let normalized_id = thing_to_string(&normalized);
            n_type = nr.kind.clone();
            n_label = format_label(nr);
            neighbors.push(Neighbor { id: normalized_id, label: n_label, node_type: n_type });
        } else {
            neighbors.push(Neighbor { id: other_id_str, label: n_label, node_type: n_type });
        }
    }

    Json(GraphNodeDetail { id: response_id, label, node_type, neighbors })
}
