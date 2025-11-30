use async_trait::async_trait;
use mnemo_core::config::providers::ProvidersConfig;
use mnemo_core::models::document::Document;

use super::docx::DocxProvider;
use super::filesystem::FilesystemProvider;
use super::github::GitHubProvider;
use super::openapi::OpenApiProvider;
use super::pdf::PdfProvider;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> String;
    fn priority(&self) -> u8;
    async fn load_documents(&self) -> Vec<Document>;
}

pub struct ProviderRegistry {
    pub providers: Vec<Box<dyn Provider + Send + Sync>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register_filesystem(&mut self, roots: Vec<std::path::PathBuf>) {
        let roots_str: Vec<String> =
            roots.into_iter().map(|p| p.to_string_lossy().to_string()).collect();
        self.providers.push(Box::new(FilesystemProvider::new(roots_str)));
        self.sort_by_priority();
    }

    pub fn register_github(&mut self, repo: String) {
        self.providers.push(Box::new(GitHubProvider::new(repo)));
        self.sort_by_priority();
    }

    pub fn register_openapi(&mut self, source: String) {
        self.providers.push(Box::new(OpenApiProvider::new(source)));
        self.sort_by_priority();
    }

    pub fn register_pdf_paths(&mut self, paths: Vec<String>) {
        self.providers.push(Box::new(PdfProvider::new(paths, "local".into())));
        self.sort_by_priority();
    }

    pub fn register_docx_paths(&mut self, paths: Vec<String>) {
        self.providers.push(Box::new(DocxProvider::new(paths, "local".into())));
        self.sort_by_priority();
    }

    /// Build providers from configuration flags.
    pub fn load_active_providers(
        &mut self,
        config: &ProvidersConfig,
    ) -> Vec<Box<dyn Provider + Send + Sync>> {
        let mut active: Vec<Box<dyn Provider + Send + Sync>> = Vec::new();
        if config.filesystem.enabled {
            let root = std::env::var("INGESTION_ROOT").unwrap_or_else(|_| "/app/data".into());
            active.push(Box::new(FilesystemProvider::new(vec![root])));
        }
        if config.github.enabled {
            active.push(Box::new(GitHubProvider::new("".into())));
        }
        if config.github.enabled {
            active.push(Box::new(OpenApiProvider::new("openapi.yaml".into())));
        }
        if config.filesystem.enabled {
            active
                .push(Box::new(PdfProvider::new(vec!["docs/example.pdf".into()], "local".into())));
            active.push(Box::new(DocxProvider::new(
                vec!["docs/example.docx".into()],
                "local".into(),
            )));
        }
        active.sort_by_key(|p| p.priority());
        active
    }

    pub fn sort_by_priority(&mut self) {
        self.providers.sort_by_key(|p| p.priority());
    }
}

#[async_trait]
impl Provider for FilesystemProvider {
    fn name(&self) -> String {
        "filesystem".into()
    }

    fn priority(&self) -> u8 {
        1
    }

    async fn load_documents(&self) -> Vec<Document> {
        self.scan()
    }
}
