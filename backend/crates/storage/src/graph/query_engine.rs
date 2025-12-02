use super::graph_engine::GraphEngine;
use crate::surreal_store::SurrealStore;
use mnemo_core::error::MnemoResult;
use serde::Serialize;
use serde_json::Value;
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
        _surreal_url: &str,
        start: &str,
        depth: usize,
    ) -> MnemoResult<GraphExpansion> {
        let store = match SurrealStore::get().await {
            Ok(c) => c,
            Err(err) => {
                tracing::error!("Surreal init failed: {err}");
                return Ok(GraphExpansion { nodes: vec![start.to_string()], edges: Vec::new() });
            }
        };

        let mut visited: HashSet<String> = HashSet::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut frontier: VecDeque<(String, usize)> = VecDeque::new();
        frontier.push_back((start.to_string(), 0));
        visited.insert(start.to_string());

        let extract_id = |v: &Value| -> Option<String> {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            }
            if let Some(obj) = v.as_object() {
                if let (Some(tb), Some(id)) =
                    (obj.get("tb").and_then(|x| x.as_str()), obj.get("id").and_then(|x| x.as_str()))
                {
                    return Some(format!("{}:{}", tb, id));
                }
            }
            None
        };

        while let Some((node, d)) = frontier.pop_front() {
            if d >= depth {
                continue;
            }
            let sql = format!(
                "SELECT in, out FROM contains WHERE in = '{}' OR out = '{}' LIMIT 100;",
                node, node
            );
            let rows = match store.select_all(&sql).await {
                Ok(r) => r,
                Err(err) => {
                    tracing::error!("Surreal expand_with_edges query failed: {err}");
                    continue;
                }
            };
            for item in rows {
                let src = item.get("out").and_then(&extract_id);
                let dst = item.get("in").and_then(&extract_id);
                let (src, dst) = match (src, dst) {
                    (Some(s), Some(d)) => (s, d),
                    _ => continue,
                };
                edges.push(GraphEdge { source: src.clone(), target: dst.clone() });
                if visited.insert(src.clone()) {
                    frontier.push_back((src, d + 1));
                }
                if visited.insert(dst.clone()) {
                    frontier.push_back((dst, d + 1));
                }
            }
        }

        Ok(GraphExpansion { nodes: visited.into_iter().collect(), edges })
    }
}
