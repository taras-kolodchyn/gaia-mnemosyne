use regex::Regex;
use unicode_normalization::UnicodeNormalization;

pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalize text to a clean, whitespace-compact form suitable for downstream chunking.
    pub fn normalize(input: &str) -> String {
        // Unicode normalize and unify line endings.
        let nfkc: String = input.nfkc().collect();
        let mut text = nfkc.replace("\r\n", "\n");

        // Strip control characters except common whitespace (tab/newline/carriage return already handled).
        text = text
            .chars()
            .filter(|c| {
                let code = *c as u32;
                !((0x0..=0x8).contains(&code) || (0x10..=0x1f).contains(&code))
            })
            .collect();

        // Trim long repeated separator characters (e.g., "-----" -> "---").
        let repeat_re = Regex::new(r"([=\-_#*]){4,}").unwrap();
        text = repeat_re
            .replace_all(&text, |caps: &regex::Captures| {
                let ch = caps.get(0).and_then(|m| m.as_str().chars().next()).unwrap_or('-');
                format!("{ch}{ch}{ch}")
            })
            .into_owned();

        // Collapse excessive whitespace while preserving paragraph breaks.
        let spaces_re = Regex::new(r"[ \t]{2,}").unwrap();
        text = spaces_re.replace_all(&text, " ").into_owned();
        let newline_re = Regex::new(r"\n{3,}").unwrap();
        text = newline_re.replace_all(&text, "\n\n").into_owned();

        text.trim().to_string()
    }
}
