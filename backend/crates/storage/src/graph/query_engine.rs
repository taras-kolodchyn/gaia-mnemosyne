use super::graph_engine::GraphEngine;
use mnemo_core::error::MnemoResult;
use reqwest::Client;
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

/// Wrapper providing a unified graph query interface.
pub struct GraphQueryEngine {
    pub engine: GraphEngine,
}

#[derive(Serialize, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Clone)]
pub struct GraphExpansion {
    pub nodes: Vec<String>,
    pub edges: Vec<GraphEdge>,
}

impl GraphQueryEngine {
    pub fn new(engine: GraphEngine) -> Self {
        Self { engine }
    }

    pub async fn expand(&self, id: &str, depth: u8) -> Vec<String> {
        self.engine.neighbors(id, depth).await
    }

    /// Expand graph using SurrealDB edges with simple BFS, returning nodes and edges.
    pub async fn expand_with_edges(
        &self,
        surreal_url: &str,
        start: &str,
        depth: usize,
    ) -> MnemoResult<GraphExpansion> {
        let client = Client::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut frontier: VecDeque<(String, usize)> = VecDeque::new();
        frontier.push_back((start.to_string(), 0));
        visited.insert(start.to_string());

        while let Some((node, d)) = frontier.pop_front() {
            if d >= depth {
                continue;
            }
            let sql = format!(
                "SELECT in, out FROM edge WHERE in = '{}' OR out = '{}' LIMIT 100;",
                node, node
            );
            let resp = client
                .post(format!("{}/sql", surreal_url))
                .body(sql)
                .send()
                .await
                .map_err(|e| mnemo_core::error::MnemoError::Message(e.to_string()))?;
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = val.get("result").and_then(|v| v.as_array()) {
                    for item in arr {
                        let src =
                            item.get("out").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let dst = item.get("in").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        if src.is_empty() || dst.is_empty() {
                            continue;
                        }
                        edges.push(GraphEdge { source: src.clone(), target: dst.clone() });
                        if visited.insert(src.clone()) {
                            frontier.push_back((src, d + 1));
                        }
                        if visited.insert(dst.clone()) {
                            frontier.push_back((dst, d + 1));
                        }
                    }
                }
            }
        }

        Ok(GraphExpansion { nodes: visited.into_iter().collect(), edges })
    }
}
