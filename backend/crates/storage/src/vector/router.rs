use std::sync::OnceLock;

fn clusters() -> &'static Vec<String> {
    static CLUSTERS: OnceLock<Vec<String>> = OnceLock::new();
    CLUSTERS.get_or_init(|| {
        let cfg = std::env::var("QDRANT_CLUSTERS").unwrap_or_else(|_| "".into());
        if cfg.trim().is_empty() {
            return vec!["http://qdrant:6333".into()];
        }
        cfg.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    })
}

fn namespace_hash(ns: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    ns.hash(&mut h);
    h.finish()
}

/// Select a Qdrant endpoint for a namespace, sharding by hash.
pub fn select_endpoint(namespace: &str) -> String {
    let list = clusters();
    if list.is_empty() {
        return "http://qdrant:6333".into();
    }
    let idx = (namespace_hash(namespace) as usize) % list.len();
    list[idx].clone()
}
