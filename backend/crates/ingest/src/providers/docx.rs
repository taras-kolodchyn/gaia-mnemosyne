use mnemo_core::models::document::Document;
use mnemo_core::utils::text_normalizer::TextNormalizer;
use sha2::{Digest, Sha256};
use std::io::Read;

/// Simple DOCX provider that extracts paragraph text from document.xml.
pub struct DocxProvider {
    pub paths: Vec<String>,
    pub namespace: String,
}

impl DocxProvider {
    pub fn new(paths: Vec<String>, namespace: String) -> Self {
        Self { paths, namespace }
    }

    pub fn load_documents(&self) -> Vec<Document> {
        let mut docs = Vec::new();
        for path in &self.paths {
            if let Ok(file) = std::fs::File::open(path) {
                if let Ok(mut zip) = zip::ZipArchive::new(file) {
                    let (author, created) = read_core_props(&mut zip);
                    if let Ok(mut doc_xml) = zip.by_name("word/document.xml") {
                        let mut xml = String::new();
                        let _ = doc_xml.read_to_string(&mut xml);
                        let text = extract_text_from_docx_xml(&xml);
                        let text = TextNormalizer::normalize(&text);
                        let mut hasher = Sha256::new();
                        hasher.update(text.as_bytes());
                        docs.push(Document {
                            path: path.clone(),
                            content: text,
                            fingerprint: format!("{:x}", hasher.finalize()),
                            namespace: self.namespace.clone(),
                            modified_at: None,
                            file_size: None,
                            file_type: Some("docx".into()),
                            language: Some("docx".into()),
                            metadata: Some(serde_json::json!({
                                "author": author,
                                "created": created
                            })),
                        });
                    }
                }
            }
        }
        docs
    }
}

fn extract_text_from_docx_xml(xml: &str) -> String {
    // DOCX paragraphs are in <w:p> with runs <w:t>. This is a simple textual scrape.
    let mut out = String::new();
    let mut inside_t = false;
    let mut buf = String::new();
    for c in xml.chars() {
        match c {
            '<' => {
                if inside_t && !buf.trim().is_empty() {
                    out.push_str(&buf);
                }
                buf.clear();
                inside_t = false;
            }
            '>' => {
                let tag = buf.clone();
                if tag.starts_with("w:t") {
                    inside_t = true;
                } else if tag.starts_with("/w:t") {
                    inside_t = false;
                    out.push(' ');
                }
                buf.clear();
            }
            _ => {
                if inside_t {
                    buf.push(c);
                } else {
                    buf.push(c);
                }
            }
        }
    }
    out.trim().to_string()
}

fn read_core_props<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
) -> (Option<String>, Option<String>) {
    if let Ok(mut core) = zip.by_name("docProps/core.xml") {
        let mut xml = String::new();
        let _ = core.read_to_string(&mut xml);
        let author = tag_value(&xml, "dc:creator");
        let created = tag_value(&xml, "dcterms:created");
        return (author, created);
    }
    (None, None)
}

fn tag_value(xml: &str, tag: &str) -> Option<String> {
    let start = format!("<{}>", tag);
    let end = format!("</{}>", tag);
    if let Some(i) = xml.find(&start) {
        if let Some(j) = xml[i + start.len()..].find(&end) {
            let val = &xml[i + start.len()..i + start.len() + j];
            return Some(val.trim().to_string());
        }
    }
    None
}

#[async_trait::async_trait]
impl super::registry::Provider for DocxProvider {
    fn name(&self) -> String {
        "docx".into()
    }

    fn priority(&self) -> u8 {
        3
    }

    async fn load_documents(&self) -> Vec<Document> {
        self.load_documents()
    }
}
