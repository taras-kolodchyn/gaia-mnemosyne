/// Compute a simple keyword relevance score between query text and a chunk of text.
/// Score = (# of query term occurrences) / (total words in chunk), clamped to 0..1.
pub fn score_keyword(query: &str, chunk: &str) -> f32 {
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if query_terms.is_empty() {
        return 0.0;
    }

    let chunk_words: Vec<String> = chunk
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if chunk_words.is_empty() {
        return 0.0;
    }

    let mut matches = 0usize;
    for w in &chunk_words {
        if query_terms.iter().any(|q| q == w) {
            matches += 1;
        }
    }

    let score = matches as f32 / chunk_words.len() as f32;
    score.clamp(0.0, 1.0)
}

/// Build a sparse keyword vector using a simple hashed bag-of-words.
/// Returns (indices, values) where indices are deterministic 32-bit hashes.
pub fn sparse_vector(text: &str) -> (Vec<u32>, Vec<f32>) {
    let mut freq: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut total = 0u32;
    for token in text
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|s| !s.is_empty())
    {
        let h = hash_token(token);
        *freq.entry(h).or_insert(0) += 1;
        total += 1;
    }
    if total == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut indices = Vec::new();
    let mut values = Vec::new();
    for (idx, count) in freq {
        let tf = count as f32 / total as f32;
        indices.push(idx);
        values.push(tf);
    }
    // Normalize values to unit length
    let norm = values.iter().map(|v| (v * v) as f64).sum::<f64>().sqrt() as f32;
    if norm > 0.0 && norm.is_finite() {
        for v in values.iter_mut() {
            *v /= norm;
            if !v.is_finite() {
                *v = 0.0;
            }
        }
    }
    (indices, values)
}

/// Deterministic 32-bit FNV-1a hash for tokens.
fn hash_token(token: &str) -> u32 {
    const FNV_OFFSET: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;
    let mut hash = FNV_OFFSET;
    for b in token.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
