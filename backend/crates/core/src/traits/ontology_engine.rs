/// Trait for classifying text into ontology tags or levels.
pub trait OntologyEngine {
    fn classify(&self, text: &str) -> Vec<String>;
}
