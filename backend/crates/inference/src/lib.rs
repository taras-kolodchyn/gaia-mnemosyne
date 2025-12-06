// Placeholder library for mnemo-inference crate.

pub mod embedding_engine;
pub mod engines;
pub mod model_router;
pub mod traits;
pub mod error;

pub use engines::tensorzero_embed::TensorZeroEmbedder;
pub use error::InferenceError;
