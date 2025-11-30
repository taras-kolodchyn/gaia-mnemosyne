use std::fs;
use std::path::PathBuf;

/// Load a fixture file from `backend/tests/fixtures`.
pub fn load_fixture(name: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = base.join("../../tests/fixtures").join(name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("fixture file missing: {}", path.display()))
}
