use axum::{
    Json,
    extract::{Path, Query},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct GraphExpandResponse {
    pub nodes: Vec<String>,
    pub edges: Vec<crate::handlers::graph_snapshot::GraphEdge>,
}

#[derive(serde::Deserialize)]
pub struct GraphExpandParams {
    pub depth: Option<usize>,
}

pub async fn graph_expand(
    Path(id): Path<String>,
    Query(params): Query<GraphExpandParams>,
) -> Json<GraphExpandResponse> {
    let depth = params.depth.unwrap_or(1).min(5);
    let surreal_url =
        std::env::var("SURREAL_URL").unwrap_or_else(|_| "http://surrealdb:8000".into());

    let store = mnemo_storage::graph::surreal::SurrealGraphStore::new(surreal_url.clone());
    let engine = mnemo_storage::graph::graph_engine::GraphEngine::new(store);
    let query_engine = mnemo_storage::graph::query_engine::GraphQueryEngine::new(engine);

    let expansion = query_engine.expand_with_edges(&surreal_url, &id, depth).await.unwrap_or(
        mnemo_storage::graph::query_engine::GraphExpansion { nodes: vec![], edges: vec![] },
    );

    let edges = expansion
        .edges
        .into_iter()
        .map(|e| {
            let id = format!("{}->{}", e.source, e.target);
            crate::handlers::graph_snapshot::GraphEdge { id, source: e.source, target: e.target }
        })
        .collect();

    Json(GraphExpandResponse { nodes: expansion.nodes, edges })
}
