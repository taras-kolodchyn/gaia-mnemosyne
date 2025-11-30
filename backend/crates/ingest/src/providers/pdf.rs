use mnemo_core::models::document::Document;
use mnemo_core::utils::text_normalizer::TextNormalizer;
use sha2::{Digest, Sha256};

/// Provider for ingesting local PDF files.
pub struct PdfProvider {
    pub paths: Vec<String>,
    pub namespace: String,
}

impl PdfProvider {
    pub fn new(paths: Vec<String>, namespace: String) -> Self {
        Self { paths, namespace }
    }

    pub fn load_documents(&self) -> Vec<Document> {
        let mut docs = Vec::new();
        for path in &self.paths {
            match std::fs::read(path) {
                Ok(bytes) => {
                    if let Ok(text) = pdf_extract::extract_text_from_mem(&bytes) {
                        let normalized = TextNormalizer::normalize(&text);
                        let pages = count_pdf_pages(&bytes);
                        let title = extract_pdf_title(&bytes);
                        let mut hasher = Sha256::new();
                        hasher.update(normalized.as_bytes());
                        docs.push(Document {
                            path: path.clone(),
                            content: normalized,
                            fingerprint: format!("{:x}", hasher.finalize()),
                            namespace: self.namespace.clone(),
                            modified_at: None,
                            file_size: Some(bytes.len() as i64),
                            file_type: Some("pdf".into()),
                            language: Some("pdf".into()),
                            metadata: Some(serde_json::json!({
                                "number_of_pages": pages,
                                "title": title
                            })),
                        });
                    }
                }
                Err(_) => continue,
            }
        }
        docs
    }
}

pub fn count_pdf_pages(bytes: &[u8]) -> usize {
    let hay = String::from_utf8_lossy(bytes).to_lowercase();
    hay.matches("/type /page").count()
}

pub fn extract_pdf_title(bytes: &[u8]) -> Option<String> {
    let hay = String::from_utf8_lossy(bytes);
    if let Some(idx) = hay.find("/Title") {
        let tail = &hay[idx..];
        if let Some(start) = tail.find('(') {
            if let Some(end) = tail[start + 1..].find(')') {
                return Some(tail[start + 1..start + 1 + end].to_string());
            }
        }
    }
    None
}

#[async_trait::async_trait]
impl super::registry::Provider for PdfProvider {
    fn name(&self) -> String {
        "pdf".into()
    }

    fn priority(&self) -> u8 {
        2
    }

    async fn load_documents(&self) -> Vec<Document> {
        self.load_documents()
    }
}
