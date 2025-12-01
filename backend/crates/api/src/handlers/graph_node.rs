use axum::{extract::Path, Json};
use mnemo_storage::surreal_rpc_client::SurrealRpcClient;
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
    let client = match SurrealRpcClient::get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("Surreal RPC init failed: {err}");
            return Json(GraphNodeDetail {
                id: id.clone(),
                label: id.clone(),
                node_type: "unknown".into(),
                neighbors: Vec::new(),
            });
        }
    };

    let node_rows = match client.query(&format!(
        "SELECT id, path, namespace, 'file' as kind FROM file WHERE id = '{id}' UNION ALL SELECT id, path, namespace, chunk_index, 'chunk' as kind FROM chunk WHERE id = '{id}';"
    )).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!("Surreal node lookup failed: {err}");
            Vec::new()
        }
    };
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

    let neighbor_rows = match client
        .query(&format!(
            "SELECT in, out FROM contains WHERE in = '{id}' OR out = '{id}';"
        ))
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!("Surreal neighbor lookup failed: {err}");
            Vec::new()
        }
    };
    tracing::info!("Graph node neighbors raw rows={}", neighbor_rows.len());

    let mut neighbors = Vec::new();
    for row in neighbor_rows {
        let in_id = row.get("in").and_then(extract_id);
        let out_id = row.get("out").and_then(extract_id);
        if let (Some(in_id), Some(out_id)) = (in_id, out_id) {
            let other = if in_id == id { out_id } else { in_id };
            let neighbor_rows = match client
                .query(&format!(
                    "SELECT id, path, namespace, 'file' as kind FROM file WHERE id = '{other}' UNION ALL SELECT id, path, namespace, chunk_index, 'chunk' as kind FROM chunk WHERE id = '{other}';"
                ))
                .await
            {
                Ok(rows) => rows,
                Err(err) => {
                    tracing::error!("Surreal neighbor detail lookup failed: {err}");
                    Vec::new()
                }
            };
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
