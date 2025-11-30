use mnemo_test_utils::fake_vector_store::FakeVectorStore;

#[tokio::test]
async fn fake_vector_store_search_works() {
    let store = FakeVectorStore;
    let results = store.search(vec![0.1, 0.2], 5).await;
    assert_eq!(results[0], "fake_result");
}
