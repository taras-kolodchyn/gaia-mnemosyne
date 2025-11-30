use crate::error::MnemoResult;
use sqlx::PgPool;

/// A single ontology rule mapping a pattern to a tag.
pub struct OntologyRule {
    pub pattern: String,
    pub tag: String,
}

/// Collection of ontology rules loaded from configuration.
pub struct OntologyRules {
    pub rules: Vec<OntologyRule>,
}

impl OntologyRules {
    pub async fn load_from_pool(pool: &PgPool) -> MnemoResult<Self> {
        let rows =
            sqlx::query("SELECT tag, patterns FROM ontology_rules").fetch_all(pool).await.map_err(
                |e| crate::error::MnemoError::Message(format!("load ontology rules failed: {e}")),
            )?;

        let mut rules = Vec::new();
        for row in rows {
            let tag: String = row.try_get("tag").unwrap_or_default();
            let patterns: Vec<String> =
                row.try_get::<Vec<String>, _>("patterns").unwrap_or_default();
            for p in patterns {
                rules.push(OntologyRule { pattern: p.to_lowercase(), tag: tag.clone() });
            }
        }
        Ok(Self { rules })
    }

    pub async fn load_from_pg_url(url: &str) -> MnemoResult<Self> {
        let pool = PgPool::connect_lazy(url)
            .map_err(|e| crate::error::MnemoError::Message(format!("pg init failed: {e}")))?;
        Self::load_from_pool(&pool).await
    }

    pub fn classify(&self, text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        let mut tags = Vec::new();
        for rule in &self.rules {
            if lower.contains(&rule.pattern) {
                tags.push(rule.tag.clone());
            }
        }
        if tags.is_empty() {
            tags.push("misc".into());
        }
        tags
    }
}
use sqlx::Row;
