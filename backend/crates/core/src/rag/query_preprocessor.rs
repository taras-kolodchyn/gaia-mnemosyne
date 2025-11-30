use std::collections::HashMap;

/// Basic query normalization and keyword extraction.
pub fn normalize(query: &str) -> String {
    query
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Very light TF-like scoring: frequency / total terms.
pub fn keywords(query: &str) -> Vec<(String, f32)> {
    let mut counts = HashMap::new();
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let total = tokens.len().max(1) as f32;
    for t in tokens {
        *counts.entry(t.to_string()).or_insert(0f32) += 1.0;
    }
    let mut scored: Vec<(String, f32)> = counts.into_iter().map(|(k, v)| (k, v / total)).collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Suggest a correction based on a small vocabulary using Levenshtein distance.
pub fn suggest_correction(token: &str, vocab: &[String]) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for v in vocab {
        let d = levenshtein(token, v);
        if d == 0 {
            return Some(v.clone());
        }
        if let Some((cur_d, _)) = &best {
            if d < *cur_d {
                best = Some((d, v.clone()));
            }
        } else {
            best = Some((d, v.clone()));
        }
    }
    best.and_then(|(d, s)| if d <= 2 { Some(s) } else { None })
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut last_cost = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let new_cost = if ca == cb { last_cost } else { last_cost + 1 };
            last_cost = costs[j + 1];
            costs[j + 1] = std::cmp::min(std::cmp::min(costs[j + 1] + 1, costs[j] + 1), new_cost);
        }
    }
    costs[b.len()]
}
