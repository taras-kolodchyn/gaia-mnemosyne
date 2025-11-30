/// Configuration for Gaia Mnemosyne.
pub struct MnemoConfig {
    pub mode: String,
    pub providers: Vec<String>,
    pub airgap: bool,
}

impl MnemoConfig {
    /// Load configuration, allowing an environment override via MNEMO_PROFILE.
    pub fn load() -> Self {
        let profile = std::env::var("MNEMO_PROFILE").unwrap_or("default".into());
        Self::load_from(&format!("config/profiles/{}.yaml", profile))
    }

    /// Placeholder loader that will eventually parse config files.
    pub fn load_from(_path: &str) -> Self {
        Self { mode: "standalone".into(), providers: vec![], airgap: false }
    }
}

pub mod ingestion_profile;
pub mod namespace;
pub mod profile_loader;
pub mod providers;
