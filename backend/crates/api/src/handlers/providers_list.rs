use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ProviderStatus {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct ProvidersResponse {
    pub filesystem: ProviderStatus,
    pub github: ProviderStatus,
}

pub async fn providers_list() -> Json<ProvidersResponse> {
    Json(ProvidersResponse {
        filesystem: ProviderStatus { enabled: true },
        github: ProviderStatus { enabled: false },
    })
}
