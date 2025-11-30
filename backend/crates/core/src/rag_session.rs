use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub query: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSession {
    pub id: Uuid,
    pub history: Vec<SessionMessage>,
}

impl RagSession {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4(), history: Vec::new() }
    }

    /// Append a message, retaining only the last 10 entries.
    pub fn append(&mut self, query: String, response: String) {
        self.history.push(SessionMessage { query, response });
        if self.history.len() > 10 {
            let len = self.history.len();
            self.history.drain(0..(len - 10));
        }
    }
}
