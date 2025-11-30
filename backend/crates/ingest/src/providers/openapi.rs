use async_trait::async_trait;
use mnemo_core::models::document::Document;
use mnemo_core::utils::text_normalizer::TextNormalizer;
use sha2::{Digest, Sha256};

/// Provider for ingesting OpenAPI specs from local files or URLs.
pub struct OpenApiProvider {
    pub source: String,
}

impl OpenApiProvider {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    fn namespace(&self) -> String {
        "openapi".to_string()
    }

    async fn load_raw(&self) -> Option<String> {
        if self.source.starts_with("http://") || self.source.starts_with("https://") {
            let client = reqwest::Client::new();
            client.get(&self.source).send().await.ok()?.text().await.ok()
        } else {
            std::fs::read_to_string(&self.source).ok()
        }
    }

    fn parse_spec(&self, raw: &str) -> Vec<Document> {
        // Try JSON first, then YAML.
        let value: serde_json::Value = serde_json::from_str(raw).unwrap_or_else(|_| {
            serde_yaml::from_str(raw)
                .map(|v: serde_yaml::Value| {
                    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
                })
                .unwrap_or(serde_json::Value::Null)
        });

        let mut docs = Vec::new();
        let Some(paths) = value.get("paths") else { return docs };
        let Some(paths_map) = paths.as_object() else { return docs };

        let namespace = self.namespace();

        for (route, entry) in paths_map {
            if let Some(methods) = entry.as_object() {
                for (method, meta) in methods {
                    let method_upper = method.to_uppercase();
                    let doc_path = format!("{} {}", method_upper, route);
                    let description = meta
                        .get("description")
                        .and_then(|v| v.as_str())
                        .or_else(|| meta.get("summary").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .to_string();

                    let mut body = String::new();
                    if !description.is_empty() {
                        body.push_str(&format!("Description: {}\n", description));
                    }

                    if let Some(params) = meta.get("parameters").and_then(|v| v.as_array()) {
                        body.push_str("Parameters:\n");
                        for p in params {
                            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let desc = p.get("description").and_then(|v| v.as_str()).unwrap_or("");
                            body.push_str(&format!("- {}: {}\n", name, desc));
                        }
                    }

                    if let Some(resps) = meta.get("responses").and_then(|v| v.as_object()) {
                        body.push_str("Responses:\n");
                        for (code, r) in resps {
                            let desc = r.get("description").and_then(|v| v.as_str()).unwrap_or("");
                            body.push_str(&format!("{}: {}\n", code, desc));
                        }
                    }

                    // capture schemas
                    if let Some(components) = value
                        .get("components")
                        .and_then(|c| c.get("schemas"))
                        .and_then(|s| s.as_object())
                    {
                        body.push_str("Schemas:\n");
                        for (name, schema) in components {
                            let summary =
                                schema.get("description").and_then(|v| v.as_str()).unwrap_or("");
                            body.push_str(&format!("{}: {}\n", name, summary));
                        }
                    }

                    let normalized = TextNormalizer::normalize(&body);
                    let body_len = normalized.len() as i64;
                    let mut hasher = Sha256::new();
                    hasher.update(normalized.as_bytes());
                    let fingerprint = format!("{:x}", hasher.finalize());

                    docs.push(Document {
                        path: doc_path,
                        content: normalized,
                        fingerprint,
                        namespace: namespace.clone(),
                        modified_at: None,
                        file_size: Some(body_len),
                        file_type: Some("openapi".to_string()),
                        language: Some("openapi".to_string()),
                        metadata: None,
                    });
                }
            }
        }

        docs
    }
}

#[async_trait]
impl super::registry::Provider for OpenApiProvider {
    fn name(&self) -> String {
        "openapi".into()
    }

    fn priority(&self) -> u8 {
        5
    }

    async fn load_documents(&self) -> Vec<Document> {
        if let Some(raw) = self.load_raw().await { self.parse_spec(&raw) } else { Vec::new() }
    }
}
