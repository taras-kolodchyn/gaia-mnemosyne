use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceConfig {
    pub vector_top_k: usize,
    pub graph_depth: u8,
    pub enable_ontology: bool,
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        Self { vector_top_k: 20, graph_depth: 1, enable_ontology: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespaceConfigs {
    pub namespaces: HashMap<String, NamespaceConfig>,
    pub default: NamespaceConfig,
}

impl NamespaceConfigs {
    pub fn load_default() -> Self {
        Self::load_from("config/namespace.yaml")
    }

    pub fn load_from(path: &str) -> Self {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_yaml::from_str::<HashMap<String, NamespaceConfig>>(&contents) {
                let default = map.get("default").cloned().unwrap_or_default();
                return Self { namespaces: map, default };
            }
        }
        Self::default()
    }

    pub fn for_namespace(&self, ns: &str) -> NamespaceConfig {
        self.namespaces.get(ns).cloned().unwrap_or_else(|| self.default.clone())
    }
}
