pub struct ProfileLoader;

impl ProfileLoader {
    pub fn load(_profile: &str) -> super::MnemoConfig {
        super::MnemoConfig { mode: "standalone".into(), providers: vec![], airgap: false }
    }
}
