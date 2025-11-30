use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct VersionResponse {
    pub version: String,
    pub git_commit: String,
    pub build_time: String,
}

pub async fn version() -> Json<VersionResponse> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let git_commit = option_env!("MNEMO_GIT_COMMIT").unwrap_or("unknown").to_string();
    let build_time = option_env!("MNEMO_BUILD_TIME").unwrap_or("0").to_string();

    Json(VersionResponse { version, git_commit, build_time })
}
