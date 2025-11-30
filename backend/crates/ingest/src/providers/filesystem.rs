use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use mnemo_core::models::document::Document;
use mnemo_core::utils::text_normalizer::TextNormalizer;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::pdf::{count_pdf_pages, extract_pdf_title};

const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &[
    "md", "txt", "json", "yaml", "yml", "rs", "toml", "pdf", "docx", "ts", "tsx", "js", "py", "go",
    "java", "rb",
];
const LARGE_THRESHOLD: usize = 200 * 1024;
const SEGMENT_SIZE: usize = 50 * 1024;

pub struct FilesystemProvider {
    pub root_paths: Vec<String>,
}

impl FilesystemProvider {
    pub fn new(root_paths: Vec<String>) -> Self {
        if root_paths.is_empty() {
            Self { root_paths: vec!["/app/data".into()] }
        } else {
            Self { root_paths }
        }
    }

    pub fn scan(&self) -> Vec<Document> {
        let mut files: Vec<(PathBuf, String)> = Vec::new();
        for root in &self.root_paths {
            let root_path = PathBuf::from(root);
            if !root_path.exists() {
                tracing::error!("Ingestion root missing: {}", root);
                return Vec::new();
            }
            tracing::info!("WalkDir scanning root: {}", root_path.display());
            collect_files(&root_path, &mut files);
        }
        tracing::info!("WalkDir collected {} file candidates", files.len());

        let mut docs = Vec::new();
        for (path, content) in files.into_iter() {
            let path_str = path.to_string_lossy().to_string();
            let normalized = TextNormalizer::normalize(&content);
            let metadata = fs::metadata(&path).ok();
            let modified_at =
                metadata.as_ref().and_then(|m| m.modified().ok()).map(|t| DateTime::<Utc>::from(t));
            let file_size = metadata.as_ref().map(|m| m.len() as i64);
            let file_type =
                path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase());
            let language = detect_language(&path_str, &normalized, file_type.as_deref());
            let mut meta: Option<serde_json::Value> = None;

            if let Some(ft) = file_type.as_deref() {
                if ft == "pdf" {
                    if let Ok(bytes) = fs::read(&path) {
                        let pages = count_pdf_pages(&bytes);
                        let title = extract_pdf_title(&bytes);
                        meta = Some(serde_json::json!({
                            "number_of_pages": pages,
                            "title": title
                        }));
                    }
                } else if ft == "docx" {
                    if let Ok(bytes) = fs::read(&path) {
                        if let Some((author, created)) = extract_docx_metadata(&bytes) {
                            meta = Some(serde_json::json!({
                                "author": author,
                                "created": created
                            }));
                        }
                    }
                }
            }

            if normalized.len() > LARGE_THRESHOLD {
                let mut start = 0;
                let mut idx = 0;
                while start < normalized.len() {
                    let end = (start + SEGMENT_SIZE).min(normalized.len());
                    let seg = &normalized[start..end];
                    let mut hasher = Sha256::new();
                    hasher.update(seg.as_bytes());
                    docs.push(Document {
                        path: format!("{}#segment_{}", path_str, idx),
                        content: seg.to_string(),
                        fingerprint: format!("{:x}", hasher.finalize()),
                        namespace: "local".to_string(),
                        modified_at,
                        file_size,
                        file_type: file_type.clone(),
                        language: language.clone(),
                        metadata: meta.clone(),
                    });
                    idx += 1;
                    start = end;
                }
            } else {
                let mut hasher = Sha256::new();
                hasher.update(normalized.as_bytes());
                docs.push(Document {
                    path: path_str,
                    content: normalized,
                    fingerprint: format!("{:x}", hasher.finalize()),
                    namespace: "local".to_string(),
                    modified_at,
                    file_size,
                    file_type: file_type.clone(),
                    language: language.clone(),
                    metadata: meta.clone(),
                });
            }
        }
        tracing::info!("FilesystemProvider produced {} documents", docs.len());
        docs
    }
}

fn detect_language(_path: &str, content: &str, ext: Option<&str>) -> Option<String> {
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return Some("python".into());
            } else if first_line.contains("node") || first_line.contains("deno") {
                return Some("javascript".into());
            } else if first_line.contains("bash") || first_line.contains("sh") {
                return Some("shell".into());
            }
        }
    }

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
            "toml" => Some("toml".into()),
            "txt" => Some("text".into()),
            "pdf" => Some("pdf".into()),
            "docx" => Some("docx".into()),
            _ => None,
        };
    }

    let text = content.to_ascii_lowercase();
    if text.contains("use std::") {
        return Some("rust".into());
    }
    if text.contains("import react") || text.contains("export default") {
        return Some("javascript".into());
    }
    if text.contains("def ") && text.contains("import ") {
        return Some("python".into());
    }

    None
}

fn collect_files(path: &Path, files: &mut Vec<(PathBuf, String)>) {
    for entry in
        WalkDir::new(path).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
    {
        let entry_path = entry.into_path();
        let Ok(metadata) = fs::metadata(&entry_path) else {
            continue;
        };

        if !is_allowed(&entry_path, metadata.len()) {
            tracing::debug!("File rejected (ext/size): {}", entry_path.display());
            continue;
        }
        tracing::info!("Found file candidate: {}", entry_path.display());

        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        if ext == "pdf" || ext == "docx" || ext == "doc" {
            tracing::debug!("Skipping binary file ({}): {}", ext, entry_path.display());
            continue;
        }

        if let Ok(bytes) = fs::read(&entry_path) {
            if !is_text(&bytes) {
                tracing::debug!("File rejected (binary): {}", entry_path.display());
                continue;
            }
            if let Ok(content) = String::from_utf8(bytes) {
                tracing::debug!("File accepted: {}", entry_path.display());
                files.push((entry_path, content));
            }
        }
    }
}

fn is_allowed(path: &Path, size: u64) -> bool {
    if size > MAX_FILE_BYTES {
        return false;
    }

    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };

    let ext_lower = ext.to_ascii_lowercase();
    ALLOWED_EXTENSIONS.contains(&ext_lower.as_str())
}

fn is_text(bytes: &[u8]) -> bool {
    !bytes.iter().any(|b| *b == 0)
}

#[allow(dead_code)]
fn extract_docx_text_from_bytes(bytes: &[u8]) -> Option<String> {
    use std::io::Cursor;
    let cursor = Cursor::new(bytes);
    if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
        if let Ok(mut doc_xml) = archive.by_name("word/document.xml") {
            let mut xml = String::new();
            let _ = doc_xml.read_to_string(&mut xml);
            return Some(extract_docx_text(&xml));
        }
    }
    None
}

#[allow(dead_code)]
fn extract_docx_text(xml: &str) -> String {
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
                if inside_t && !buf.trim().is_empty() {
                    out.push(' ');
                    out.push_str(&buf);
                }
                buf.clear();
                inside_t = true;
            }
            _ => buf.push(c),
        }
    }
    out
}

fn extract_docx_metadata(bytes: &[u8]) -> Option<(String, String)> {
    use std::io::Cursor;
    let cursor = Cursor::new(bytes);
    if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
        if let Ok(mut core_xml) = archive.by_name("docProps/core.xml") {
            let mut xml = String::new();
            let _ = core_xml.read_to_string(&mut xml);
            let author = extract_tag(&xml, "dc:creator").unwrap_or_default();
            let created = extract_tag(&xml, "dcterms:created").unwrap_or_default();
            return Some((author, created));
        }
    }
    None
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let start = format!("<{tag}>");
    let end = format!("</{tag}>");
    let s = xml.find(&start)?;
    let e = xml[s + start.len()..].find(&end)?;
    Some(xml[s + start.len()..s + start.len() + e].to_string())
}
