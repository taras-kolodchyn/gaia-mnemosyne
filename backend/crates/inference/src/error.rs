use thiserror::Error;

#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Request failed: {0}")]
    Status(String),
    #[error("Deserialization failed: {0}")]
    Deserialize(String),
    #[error("Other: {0}")]
    Other(String),
}

impl InferenceError {
    pub fn msg(msg: impl Into<String>) -> Self {
        InferenceError::Other(msg.into())
    }
}

impl From<reqwest::Error> for InferenceError {
    fn from(err: reqwest::Error) -> Self {
        InferenceError::Http(err.to_string())
    }
}

impl From<serde_json::Error> for InferenceError {
    fn from(err: serde_json::Error) -> Self {
        InferenceError::Deserialize(err.to_string())
    }
}
