use regex::Regex;

/// File types used for tailored chunking.
#[derive(Clone, Copy, Debug)]
pub enum FileType {
    Markdown,
    Code,
    Text,
    Json,
    Yaml,
    Pdf,
    Docx,
    Unknown,
}

pub struct ChunkBuilder;

impl ChunkBuilder {
    /// Detect file type based on extension.
    pub fn detect(path: &str) -> FileType {
        let base = path.split('#').next().unwrap_or(path);
        let ext = std::path::Path::new(base)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "md" | "markdown" => FileType::Markdown,
            "rs" | "ts" | "tsx" | "js" | "py" | "go" | "java" | "rb" | "cpp" | "c" => {
                FileType::Code
            }
            "json" => FileType::Json,
            "yaml" | "yml" => FileType::Yaml,
            "txt" => FileType::Text,
            "pdf" => FileType::Pdf,
            "docx" => FileType::Docx,
            _ => FileType::Unknown,
        }
    }

    pub fn detect_language(language: &str) -> FileType {
        match language.to_ascii_lowercase().as_str() {
            "markdown" => FileType::Markdown,
            "rust" | "typescript" | "javascript" | "python" | "go" | "java" | "ruby" | "c"
            | "cpp" => FileType::Code,
            "json" => FileType::Json,
            "yaml" | "yml" => FileType::Yaml,
            "text" => FileType::Text,
            _ => FileType::Unknown,
        }
    }

    /// Build chunks based on file type heuristics.
    pub fn build(text: &str, file_type: FileType) -> Vec<String> {
        let mut raw = match file_type {
            FileType::Markdown => Self::markdown_chunks(text),
            FileType::Code => Self::code_chunks(text),
            FileType::Text => Self::text_chunks(text),
            FileType::Json => Self::json_chunks(text),
            FileType::Yaml => Self::yaml_chunks(text),
            FileType::Pdf => Self::pdf_chunks(text),
            FileType::Docx => Self::docx_chunks(text),
            FileType::Unknown => vec![text.to_string()],
        };

        // Normalize into ~800-1200 char segments.
        raw = Self::segment(raw, 800, 1200);
        // Drop empties.
        raw.into_iter().filter(|c| !c.trim().is_empty()).collect()
    }

    fn segment(chunks: Vec<String>, min_len: usize, max_len: usize) -> Vec<String> {
        let mut out = Vec::new();
        for ch in chunks {
            if ch.len() <= max_len {
                out.push(ch);
                continue;
            }
            let mut start = 0;
            while start < ch.len() {
                let end = (start + max_len).min(ch.len());
                let slice = &ch[start..end];
                if slice.len() >= min_len {
                    out.push(slice.to_string());
                }
                start = end;
            }
        }
        out
    }

    fn pdf_chunks(text: &str) -> Vec<String> {
        // PDFs often have form feeds or page markers; split on page breaks first.
        let pages: Vec<&str> = text.split('\u{0c}').collect();
        let mut chunks = Vec::new();
        for page in pages {
            for para in page.split("\n\n") {
                let p = para.trim();
                if !p.is_empty() {
                    chunks.push(p.to_string());
                }
            }
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }

    fn docx_chunks(text: &str) -> Vec<String> {
        // Treat each blank-line separated block as a paragraph.
        let mut chunks = Vec::new();
        for para in text.split("\n\n") {
            let p = para.trim();
            if !p.is_empty() {
                chunks.push(p.to_string());
            }
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }

    fn markdown_chunks(text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();
        for line in text.lines() {
            if line.starts_with('#') && !current.trim().is_empty() {
                chunks.push(current.trim().to_string());
                current.clear();
            }
            current.push_str(line);
            current.push('\n');
        }
        if !current.trim().is_empty() {
            chunks.push(current.trim().to_string());
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }

    fn code_chunks(text: &str) -> Vec<String> {
        let re = Regex::new(r"(?m)^(pub\s+fn\s|fn\s|impl\s|def\s|class\s)").unwrap();
        let mut chunks = Vec::new();
        let mut current = String::new();
        for line in text.lines() {
            if re.is_match(line) && !current.trim().is_empty() {
                chunks.push(current.trim().to_string());
                current.clear();
            }
            current.push_str(line);
            current.push('\n');
        }
        if !current.trim().is_empty() {
            chunks.push(current.trim().to_string());
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }

    fn text_chunks(text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }

    fn json_chunks(text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(obj) = val.as_object() {
                for (k, v) in obj {
                    let snippet = serde_json::to_string_pretty(v).unwrap_or_default();
                    chunks.push(format!("key: {}\n{}", k, snippet));
                }
            }
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }

    fn yaml_chunks(text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(text) {
            if let Some(obj) = val.as_mapping() {
                for (k, v) in obj {
                    let key = serde_yaml::to_string(&k).unwrap_or_default();
                    let val_str = serde_yaml::to_string(&v).unwrap_or_default();
                    chunks.push(format!("key: {}\n{}", key.trim(), val_str.trim()));
                }
            }
        }
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }
        chunks
    }
}
