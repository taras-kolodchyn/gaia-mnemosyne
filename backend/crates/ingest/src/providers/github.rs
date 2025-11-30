use async_trait::async_trait;
use mnemo_core::models::document::Document;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LARGE_THRESHOLD: usize = 200 * 1024;
const SEGMENT_SIZE: usize = 50 * 1024;

/// GitHub provider that fetches repository files via GitHub REST v3.
pub struct GitHubProvider {
    pub repo: String,
}

impl GitHubProvider {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }

    fn list_files_recursive<'a>(
        &'a self,
        client: &'a reqwest::Client,
        owner: &'a str,
        repo: &'a str,
        path: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<(String, String)>> + Send + 'a>>
    {
        Box::pin(async move {
            let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
            let mut attempt = 0;
            let res = loop {
                let resp = client.get(&url).send().await;
                let Ok(resp) = resp else { return Vec::new() };
                if resp.status() == reqwest::StatusCode::FORBIDDEN {
                    if let Some(reset) = resp
                        .headers()
                        .get("X-RateLimit-Reset")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                    {
                        if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                            let wait_secs = reset.saturating_sub(now.as_secs()).max(1);
                            tracing::warn!("GitHub rate limit hit, sleeping {}s", wait_secs);
                            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                            continue;
                        }
                    }
                    let backoff = Duration::from_millis((2_u64.pow(attempt)).min(8000));
                    tracing::warn!("GitHub 403, backoff {:?} attempt {}", backoff, attempt);
                    tokio::time::sleep(backoff).await;
                    attempt = attempt.saturating_add(1);
                    continue;
                }
                break resp;
            };
            let Ok(entries) = res.json::<serde_json::Value>().await else { return Vec::new() };
            let mut files = Vec::new();
            if let Some(arr) = entries.as_array() {
                for item in arr {
                    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let item_path =
                        item.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if item_type == "file" {
                        if let Some(download_url) =
                            item.get("download_url").and_then(|v| v.as_str())
                        {
                            if let Ok(resp) = client.get(download_url).send().await {
                                if let Ok(content) = resp.text().await {
                                    // rudimentary binary check: skip if contains null bytes
                                    if !content.as_bytes().iter().any(|b| *b == 0) {
                                        files.push((item_path, content));
                                    }
                                }
                            }
                        }
                    } else if item_type == "dir" {
                        let nested =
                            self.list_files_recursive(client, owner, repo, &item_path).await;
                        for n in nested {
                            files.push(n);
                        }
                    }
                }
            }
            files
        })
    }
}

#[async_trait]
impl super::registry::Provider for GitHubProvider {
    fn name(&self) -> String {
        "github".into()
    }

    fn priority(&self) -> u8 {
        10
    }

    async fn load_documents(&self) -> Vec<Document> {
        let token = std::env::var("GITHUB_TOKEN").unwrap_or_default();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("gaia-mnemosyne"),
        );
        if !token.is_empty() {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(&format!("token {}", token)) {
                headers.insert(reqwest::header::AUTHORIZATION, value);
            }
        }

        let client = reqwest::Client::builder().default_headers(headers).build();
        let Ok(client) = client else { return Vec::new() };

        let parts: Vec<&str> = self.repo.split('/').collect();
        if parts.len() != 2 {
            return Vec::new();
        }
        let owner = parts[0];
        let repo = parts[1];

        self.list_files_recursive(&client, owner, repo, "")
            .await
            .into_iter()
            .map(|(path, content)| {
                let mut docs = Vec::new();
                let file_size = Some(content.len() as i64);
                let file_type = path.split('.').last().map(|s| s.to_ascii_lowercase());
                let language = detect_language(&path, &content, file_type.as_deref());
                let metadata = None;
                if content.len() > LARGE_THRESHOLD {
                    let mut start = 0;
                    let mut idx = 0;
                    while start < content.len() {
                        let end = (start + SEGMENT_SIZE).min(content.len());
                        let seg = content[start..end].to_string();
                        let mut hasher = Sha256::new();
                        hasher.update(seg.as_bytes());
                        docs.push(Document {
                            path: format!("{}#segment_{}", path, idx),
                            content: seg,
                            fingerprint: format!("{:x}", hasher.finalize()),
                            namespace: self.repo.clone(),
                            modified_at: None,
                            file_size,
                            file_type: file_type.clone(),
                            language: language.clone(),
                            metadata: metadata.clone(),
                        });
                        idx += 1;
                        start = end;
                    }
                } else {
                    let mut hasher = Sha256::new();
                    hasher.update(content.as_bytes());
                    docs.push(Document {
                        path,
                        content,
                        fingerprint: format!("{:x}", hasher.finalize()),
                        namespace: self.repo.clone(),
                        modified_at: None,
                        file_size,
                        file_type: file_type.clone(),
                        language: language.clone(),
                        metadata: metadata.clone(),
                    });
                }
                docs
            })
            .flatten()
            .collect()
    }
}

fn detect_language(path: &str, content: &str, ext: Option<&str>) -> Option<String> {
    if let Some(e) = ext {
        return match e {
            "rs" => Some("rust".into()),
            "ts" | "tsx" => Some("typescript".into()),
            "js" => Some("javascript".into()),
            "py" => Some("python".into()),
            "go" => Some("go".into()),
            "java" => Some("java".into()),
            "rb" => Some("ruby".into()),
            "md" => Some("markdown".into()),
            "json" => Some("json".into()),
            "yaml" | "yml" => Some("yaml".into()),
            _ => None,
        };
    }

    let text = content.to_ascii_lowercase();
    if text.contains("use std::") {
        return Some("rust".into());
    }
    if text.contains("import React") || text.contains("export default") {
        return Some("javascript".into());
    }
    if text.contains("def ") && text.contains("import ") {
        return Some("python".into());
    }
    if path.contains("openapi") || path.contains("swagger") {
        return Some("openapi".into());
    }
    None
}
