use crate::traits::keyword_search::KeywordSearch;

pub struct DefaultKeywordSearch;

impl KeywordSearch for DefaultKeywordSearch {
    fn search(&self, query: &str) -> Vec<String> {
        vec![format!("keyword_match_for_{}", query)]
    }
}
