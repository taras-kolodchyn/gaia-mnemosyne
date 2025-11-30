use serde::{Deserialize, Serialize};

use crate::models::graph_nodes::{ConceptNode, FileNode, RepoNode};

#[derive(Clone, Debug, Deserialize)]
pub struct SurrealRecord {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: String,
}

pub fn map_surreal_record_to_node(record: SurrealRecord) -> GraphNode {
    let kind = if record.id.starts_with("repo:") {
        RepoNode { name: record.label.clone().unwrap_or_else(|| record.id.clone()) }.kind()
    } else if record.id.starts_with("file:") {
        FileNode { path: record.label.clone().unwrap_or_else(|| record.id.clone()) }.kind()
    } else if record.id.starts_with("chunk:") {
        ConceptNode { label: record.label.clone().unwrap_or_else(|| record.id.clone()) }.kind()
    } else {
        "unknown".into()
    };

    let label = record.label.unwrap_or_else(|| record.id.clone());

    GraphNode { id: record.id, label, kind }
}
