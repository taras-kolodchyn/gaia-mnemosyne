use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Deserialize, Default)]
struct ModelEntry {
    model: Option<String>,
}

#[derive(Deserialize, Default)]
struct RouterConfig {
    #[serde(default)]
    default: ModelEntry,
    #[serde(default)]
    rust: Option<ModelEntry>,
    #[serde(default)]
    markdown: Option<ModelEntry>,
    #[serde(default)]
    openapi: Option<ModelEntry>,
}

fn default_model_alias() -> Option<String> {
    let from_single = std::env::var("TENSORZERO_EMBED_MODEL")
        .or_else(|_| std::env::var("MNEMO_EMBED_MODEL"))
        .ok()
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
    if from_single.is_some() {
        return from_single;
    }

    std::env::var("TENSORZERO_EMBED_MODELS")
        .or_else(|_| std::env::var("MNEMO_EMBED_MODELS"))
        .ok()
        .and_then(|list| {
            list.split(',')
                .map(|s| s.trim())
                .find(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
}

fn load_config_map() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let path =
            std::env::var("MODEL_ROUTER_CONFIG").unwrap_or_else(|_| "model_router.toml".into());
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<RouterConfig>(&raw) {
                let mut map = HashMap::new();
                if let Some(m) = cfg.default.model {
                    map.insert("default".into(), m);
                }
                if let Some(entry) = cfg.rust.and_then(|e| e.model) {
                    map.insert("rust".into(), entry);
                }
                if let Some(entry) = cfg.markdown.and_then(|e| e.model) {
                    map.insert("markdown".into(), entry);
                }
                if let Some(entry) = cfg.openapi.and_then(|e| e.model) {
                    map.insert("openapi".into(), entry);
                }
                return map;
            }
        }

        if let Some(default) = default_model_alias() {
            return HashMap::from([
                ("default".into(), default.clone()),
                ("rust".into(), default.clone()),
                ("markdown".into(), default.clone()),
                ("openapi".into(), default),
            ]);
        }

        HashMap::new()
    })
}

/// Select an embedding model based on file type, namespace, language and size.
pub fn select_model(
    file_type: Option<&str>,
    namespace: Option<&str>,
    language: Option<&str>,
    doc_size: Option<i64>,
) -> String {
    let map = load_config_map();
    let default = map
        .get("default")
        .cloned()
        .or_else(default_model_alias)
        .unwrap_or_default();

    // Large documents -> document-focused model if configured.
    if let Some(size) = doc_size {
        if size > 200_000 {
            if let Some(m) = map.get("markdown") {
                return m.clone();
            }
        }
    }

    let ft = file_type.map(|f| f.to_ascii_lowercase()).unwrap_or_default();
    let lang = language.map(|l| l.to_ascii_lowercase()).unwrap_or_default();
    let ns = namespace.unwrap_or("").to_ascii_lowercase();

    if ft.contains("openapi") {
        if let Some(m) = map.get("openapi") {
            return m.clone();
        }
    }

    if ft.ends_with("md") || ft.contains("markdown") {
        if let Some(m) = map.get("markdown") {
            return m.clone();
        }
    }

    if ft.ends_with("rs") || ft.contains("rust") || lang == "rust" || ns.contains("rust") {
        if let Some(m) = map.get("rust") {
            return m.clone();
        }
    }

    default
}
