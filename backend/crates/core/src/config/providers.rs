use std::collections::HashMap;

/// Configuration for a single provider.
pub struct ProviderConfig {
    pub enabled: bool,
    pub options: HashMap<String, String>,
}

/// Aggregated provider configuration.
pub struct ProvidersConfig {
    pub filesystem: ProviderConfig,
    pub github: ProviderConfig,
}

impl ProvidersConfig {
    /// Placeholder loader until real profile parsing is added.
    pub fn load() -> Self {
        Self {
            filesystem: ProviderConfig { enabled: false, options: HashMap::new() },
            github: ProviderConfig { enabled: false, options: HashMap::new() },
        }
    }
}
