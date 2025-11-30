use axum::{Json, extract::Path};
use serde::Serialize;

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
        if let Ok(val) = resp.json::<serde_json::Value>().await {
            if let Some(arr) = val.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
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
            "SELECT id, path, namespace, 'file' as kind FROM file WHERE id = '{id}' UNION ALL SELECT id, path, namespace, 'chunk' as kind FROM chunk WHERE id = '{id}';"
        ),
        &surreal_url,
        &ns,
        &db,
        &user,
        &pass,
    )
    .await;

    let mut node_type = "unknown".to_string();
    let mut label = id.clone();
    if let Some(row) = node_rows.get(0) {
        if let Some(k) = row.get("kind").and_then(|v| v.as_str()) {
            node_type = k.to_string();
        }
        if let Some(p) = row.get("path").and_then(|v| v.as_str()) {
            label = p.to_string();
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

    let mut neighbors = Vec::new();
    for row in neighbor_rows {
        let in_id = row.get("in").and_then(|v| v.as_str());
        let out_id = row.get("out").and_then(|v| v.as_str());
        if let (Some(in_id), Some(out_id)) = (in_id, out_id) {
            let other = if in_id == id { out_id } else { in_id };
            neighbors.push(Neighbor {
                id: other.to_string(),
                label: other.to_string(),
                node_type: "unknown".into(),
            });
        }
    }

    Json(GraphNodeDetail { id: id.clone(), label, node_type, neighbors })
}
