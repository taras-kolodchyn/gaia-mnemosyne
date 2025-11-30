/// Simple k-means placeholder for document centroids.
/// Input: Vec<(doc_id, embedding)>.
/// Returns Vec<(doc_id, cluster_id)>.
pub fn cluster_documents(centroids: &[(String, Vec<f32>)], k: usize) -> Vec<(String, i32)> {
    if k == 0 || centroids.is_empty() {
        return Vec::new();
    }

    // Very naive: assign by round-robin for now; replace with real k-means later.
    centroids.iter().enumerate().map(|(idx, (doc, _))| (doc.clone(), (idx % k) as i32)).collect()
}
