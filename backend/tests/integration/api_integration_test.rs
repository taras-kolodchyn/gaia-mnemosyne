use mnemo_api::build_router;

#[tokio::test]
async fn router_initializes() {
    let _router = build_router();
    assert!(true);
}
