use axum::Router;
use mnemo_api::build_router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app: Router = build_router();
    let app = app.layer(CorsLayer::permissive());
    let port = std::env::var("MNEMO_HTTP_PORT").unwrap_or("7700".into());
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting Gaia Mnemosyne API service on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
