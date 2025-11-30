use serde::Deserialize;

#[derive(Deserialize)]
pub struct ContextQueryRequest {
    pub query: String,
}
