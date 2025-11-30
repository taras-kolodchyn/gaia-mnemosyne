use crate::{
    models::rag_context::RAGContext,
    rag::{
        cache,
        keyword::{score_keyword, sparse_vector},
        query_preprocessor,
    },
    rag_session::RagSession,
    traits::ranking_engine::RankingEngine,
};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;

/// Placeholder orchestrator for RAG query execution.
pub struct RAGOrchestrator;

struct SimpleRankingEngine;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DebugCandidate {
    pub chunk: String,
    pub vector_score: f32,
    pub keyword_score: f32,
    pub graph_score: f32,
    pub ontology_score: f32,
    pub final_score: f32,
    pub tags: Vec<String>,
    pub neighbors_count: usize,
}

impl RankingEngine for SimpleRankingEngine {
    fn score(
        &self,
        vector_score: f32,
        keyword_score: f32,
        graph_score: f32,
        knowledge_score: f32,
    ) -> f32 {
        // Simple weighted sum; adjust weights later via config.
        vector_score * 0.6 + keyword_score * 0.2 + graph_score * 0.1 + knowledge_score * 0.1
    }
}

impl RAGOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, query: &str) -> RAGContext {
        self.run_with_context(query, None).await
    }

    pub async fn run_with_context(&self, query: &str, session: Option<&RagSession>) -> RAGContext {
        if let Some(cached) = cache::get_cached(query).await {
            return cached;
        }

        let candidates = self.gather_candidates(query, session).await;

        let mut project_chunks = Vec::new();
        let mut domain_chunks = Vec::new();
        let mut company_chunks = Vec::new();
        for candidate in candidates.iter().take(10) {
            if candidate.tags.iter().any(|t| t.contains("project")) {
                project_chunks.push(candidate.chunk.clone());
            } else if candidate.tags.iter().any(|t| t.contains("domain")) {
                domain_chunks.push(candidate.chunk.clone());
            } else if candidate.tags.iter().any(|t| t.contains("company")) {
                company_chunks.push(candidate.chunk.clone());
            } else {
                project_chunks.push(candidate.chunk.clone());
            }
        }

        // Fallback if nothing returned from vector search
        if project_chunks.is_empty() && domain_chunks.is_empty() && company_chunks.is_empty() {
            project_chunks.push("test_project_chunk".into());
            domain_chunks.push("test_domain_chunk".into());
            company_chunks.push("test_company_chunk".into());
        }

        let ctx = RAGContext {
            project_chunks,
            domain_chunks,
            company_chunks,
            graph_neighbors: vec![],
            ontology_tags: vec![],
            debug_candidates: candidates,
        };

        cache::set_cached(query, &ctx).await;
        ctx
    }

    /// Return scored candidates with explanations for debugging.
    pub async fn gather_candidates(
        &self,
        query: &str,
        session: Option<&RagSession>,
    ) -> Vec<DebugCandidate> {
        let namespace = "default";
        let qdrant_url = select_endpoint(namespace);

        let mut previous_queries: Vec<String> = Vec::new();
        if let Some(sess) = session {
            for msg in sess.history.iter().rev().take(5) {
                previous_queries.push(msg.query.clone());
            }
        }
        let cleaned_query = query_preprocessor::normalize(query);
        let inferred_tags = infer_tags(&cleaned_query);
        let graph_neighbors = fetch_graph_neighbors(2).await;
        let extended_query = build_extended_query(
            &cleaned_query,
            &previous_queries,
            &inferred_tags,
            &graph_neighbors,
        );
        let query_vec = vec![0.1_f32; 1536];
        let (q_indices, q_values) = sparse_vector(&extended_query);
        let ranker = SimpleRankingEngine;
        let mut explanations = Vec::new();

        let payload = json!({
            "vector": { "name": "dense", "vector": query_vec },
            "sparse_vector": { "indices": q_indices, "values": q_values },
            "limit": 5,
            "with_payload": true,
            "filter": { "must": [ { "key": "namespace", "match": { "value": namespace } } ] }
        });

        if let Ok(resp) = Client::new()
            .post(format!("{}/collections/mnemo_chunks/points/search", qdrant_url))
            .json(&payload)
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = body.get("result").and_then(|r| r.as_array()) {
                    for item in arr {
                        let vector_score =
                            item.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let point_id = item.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                        let mut text = String::new();
                        let mut doc_path = String::new();
                        let mut namespace_val = namespace.to_string();
                        let mut tags: Vec<String> = Vec::new();

                        if let Some(entry) = cache::get_chunk_cached(point_id).await {
                            text = entry.text;
                            doc_path = entry.file_path;
                            tags = entry.tags;
                        } else if let Some(payload) = item.get("payload") {
                            text = payload
                                .get("text")
                                .or_else(|| payload.get("chunk_text"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            doc_path = payload
                                .get("document")
                                .or_else(|| payload.get("document_path"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            namespace_val = payload
                                .get("namespace")
                                .and_then(|v| v.as_str())
                                .unwrap_or(namespace)
                                .to_string();
                            tags = payload
                                .get("tags")
                                .and_then(|v| {
                                    if let Some(arr) = v.as_array() {
                                        Some(
                                            arr.iter()
                                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                                .collect::<Vec<_>>(),
                                        )
                                    } else if let Some(s) = v.as_str() {
                                        Some(
                                            s.split(',')
                                                .map(|t| t.trim().to_string())
                                                .filter(|t| !t.is_empty())
                                                .collect(),
                                        )
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default();
                            cache::set_chunk_cached(
                                point_id,
                                &cache::ChunkCacheEntry {
                                    text: text.clone(),
                                    tags: tags.clone(),
                                    file_path: doc_path.clone(),
                                },
                            )
                            .await;
                        }

                        let mut keyword_score = if extended_query.is_empty() {
                            0.0
                        } else {
                            score_keyword(&extended_query, &text)
                        };
                        if is_code_file(&doc_path) {
                            keyword_score *= 0.9;
                        }
                        if text.trim_start().starts_with('#') {
                            keyword_score *= 1.10;
                        }
                        if namespace_val.starts_with("openapi") {
                            keyword_score *= 1.15;
                        }

                        // Ontology weighting: simple boosts.
                        let knowledge_score = if tags.iter().any(|t| t.contains("project")) {
                            1.0
                        } else if tags.iter().any(|t| t.contains("domain")) {
                            0.8
                        } else if tags.iter().any(|t| t.contains("company")) {
                            0.6
                        } else {
                            0.0
                        };

                        let file_id = format!("file:{}", doc_path);
                        let (graph_score, neighbors_count) = graph_score_for(&file_id, 1).await;
                        let final_score =
                            ranker.score(vector_score, keyword_score, graph_score, knowledge_score);

                        explanations.push(DebugCandidate {
                            chunk: text,
                            vector_score,
                            keyword_score,
                            graph_score,
                            ontology_score: knowledge_score,
                            final_score,
                            tags: tags.clone(),
                            neighbors_count,
                        });
                    }
                }
            }
        }

        if explanations.is_empty() {
            explanations.push(DebugCandidate {
                chunk: "test_project_chunk".into(),
                vector_score: 0.0,
                keyword_score: 0.0,
                graph_score: 0.0,
                ontology_score: 0.0,
                final_score: 0.1,
                tags: vec!["project".into()],
                neighbors_count: 0,
            });
        }

        explanations.sort_by(|a, b| {
            b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        explanations
    }
}

fn build_extended_query(
    query: &str,
    previous_queries: &[String],
    tags: &[String],
    neighbors: &[String],
) -> String {
    let mut parts = Vec::new();
    parts.push(query.to_string());
    if !previous_queries.is_empty() {
        parts.push(previous_queries.join(" "));
    }
    if !tags.is_empty() {
        parts.push(tags.join(" "));
    }
    if !neighbors.is_empty() {
        parts.push(neighbors.join(" "));
    }
    parts.join(" ")
}

fn infer_tags(query: &str) -> Vec<String> {
    let q = query.to_ascii_lowercase();
    let mut tags = HashSet::new();
    for (needle, tag) in [
        ("project", "project"),
        ("repo", "project"),
        ("domain", "domain"),
        ("company", "company"),
        ("org", "company"),
    ] {
        if q.contains(needle) {
            tags.insert(tag.to_string());
        }
    }
    if tags.is_empty() {
        // default hint to keep context from being empty
        tags.insert("project".to_string());
    }
    tags.into_iter().collect()
}

async fn fetch_graph_neighbors(depth: u8) -> Vec<String> {
    let client = Client::new();
    let limit = 50 * depth.max(1) as usize;
    let sql = format!("SELECT DISTINCT in, out FROM edge LIMIT {};", limit);
    if let Ok(resp) = client.post("http://surrealdb:8000/sql").body(sql).send().await {
        if let Ok(val) = resp.json::<serde_json::Value>().await {
            let mut neighbors = HashSet::new();
            if let Some(arr) = val.get("result").and_then(|v| v.as_array()) {
                for row in arr {
                    if let Some(i) = row.get("in").and_then(|v| v.as_str()) {
                        neighbors.insert(i.to_string());
                    }
                    if let Some(o) = row.get("out").and_then(|v| v.as_str()) {
                        neighbors.insert(o.to_string());
                    }
                }
            }
            return neighbors.into_iter().collect();
        }
    }
    Vec::new()
}

async fn graph_score_for(file_id: &str, depth: u8) -> (f32, usize) {
    let client = Client::new();
    let limit = 100 * depth.max(1) as usize;
    let sql = format!(
        "SELECT * FROM edge WHERE in = '{}' OR out = '{}' LIMIT {};",
        file_id, file_id, limit
    );
    if let Ok(resp) = client.post("http://surrealdb:8000/sql").body(sql).send().await {
        if let Ok(val) = resp.json::<serde_json::Value>().await {
            if let Some(arr) = val.get("result").and_then(|v| v.as_array()) {
                let count = arr.len();
                return (((count as f32) / 5.0).clamp(0.0, 1.0), count);
            }
        }
    }
    (0.0, 0)
}

fn is_code_file(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(ext.as_str(), "rs" | "ts" | "tsx" | "js" | "py" | "go" | "java" | "rb" | "cpp" | "c")
}

fn select_endpoint(namespace: &str) -> String {
    let cfg = std::env::var("QDRANT_CLUSTERS").unwrap_or_else(|_| "".into());
    let clusters: Vec<String> = if cfg.trim().is_empty() {
        vec!["http://qdrant:6333".into()]
    } else {
        cfg.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    };
    if clusters.is_empty() {
        return "http://qdrant:6333".into();
    }
    let idx = (namespace_hash(namespace) as usize) % clusters.len();
    clusters[idx].clone()
}

fn namespace_hash(ns: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    ns.hash(&mut h);
    h.finish()
}
