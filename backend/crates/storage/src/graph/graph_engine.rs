use super::surreal::SurrealGraphStore;

/// Placeholder graph engine wrapper.
pub struct GraphEngine {
    pub store: SurrealGraphStore,
}

impl GraphEngine {
    pub fn new(store: SurrealGraphStore) -> Self {
        Self { store }
    }

    pub async fn neighbors(&self, _id: &str, _depth: u8) -> Vec<String> {
        vec!["graph_neighbor_placeholder".to_string()]
    }
}
