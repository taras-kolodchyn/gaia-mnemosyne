/// Represents a repository node in the knowledge graph.
pub struct RepoNode {
    pub name: String,
}

impl RepoNode {
    pub fn label(&self) -> String {
        self.name.clone()
    }

    pub fn kind(&self) -> String {
        "repo".into()
    }
}

/// Represents a file node in the knowledge graph.
pub struct FileNode {
    pub path: String,
}

impl FileNode {
    pub fn label(&self) -> String {
        self.path.clone()
    }

    pub fn kind(&self) -> String {
        "file".into()
    }
}

/// Represents a concept node in the knowledge graph.
pub struct ConceptNode {
    pub label: String,
}

impl ConceptNode {
    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn kind(&self) -> String {
        "chunk".into()
    }
}
