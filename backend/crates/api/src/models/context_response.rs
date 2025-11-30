use serde::Serialize;

#[derive(Serialize)]
pub struct ContextQueryResponse {
    pub project_chunks: Vec<String>,
    pub domain_chunks: Vec<String>,
    pub company_chunks: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}
