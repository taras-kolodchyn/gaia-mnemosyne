/// Placeholder SQLite-backed metadata store.
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Clone)]
pub struct SQLiteMetadataStore {
    pub path: String,
}

static FP_CACHE: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

impl SQLiteMetadataStore {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn has_fingerprint(&self, fp: &str) -> bool {
        FP_CACHE.lock().map(|set| set.contains(fp)).unwrap_or(false)
    }

    pub fn store_fingerprint(&self, fp: &str) {
        if let Ok(mut set) = FP_CACHE.lock() {
            set.insert(fp.to_string());
        }
    }
}
