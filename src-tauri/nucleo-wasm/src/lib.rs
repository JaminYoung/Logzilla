use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct MatchResult {
    pub line_index: usize,
    pub score: i32,
    pub highlights: Vec<usize>,
}

#[wasm_bindgen]
pub struct NucleoSearch {
    matcher: Matcher,
}

#[wasm_bindgen]
impl NucleoSearch {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Perform fuzzy search on lines
    pub fn search(
        &mut self,
        query: &str,
        lines: &JsValue,
        case_sensitive: bool,
        limit: usize,
    ) -> JsValue {
        let lines: Vec<String> = serde_wasm_bindgen::from_value(lines.clone()).unwrap_or_default();

        if query.is_empty() {
            let results: Vec<MatchResult> = Vec::new();
            return serde_wasm_bindgen::to_value(&results).unwrap();
        }

        let case = if case_sensitive {
            CaseMatching::Respect
        } else if query.chars().any(|c| c.is_uppercase()) {
            CaseMatching::Respect
        } else {
            CaseMatching::Ignore
        };

        let pattern = Pattern::new(query, case, Normalization::Smart, AtomKind::Fuzzy);
        let mut char_buf = Vec::new();
        let mut indices_buf: Vec<u32> = Vec::new();

        let mut results: Vec<MatchResult> = lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                char_buf.clear();
                indices_buf.clear();
                let haystack = Utf32Str::new(line, &mut char_buf);
                let score = pattern.score(haystack, &mut self.matcher)?;
                pattern.indices(haystack, &mut self.matcher, &mut indices_buf);

                Some(MatchResult {
                    line_index: idx,
                    score: score as i32,
                    highlights: indices_buf.iter().map(|&i| i as usize).collect(),
                })
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Limit results
        results.truncate(limit);

        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    /// Perform regex search on lines
    pub fn search_regex(
        &mut self,
        pattern: &str,
        lines: &JsValue,
        case_sensitive: bool,
        limit: usize,
    ) -> JsValue {
        let lines: Vec<String> = serde_wasm_bindgen::from_value(lines.clone()).unwrap_or_default();

        let flags = if case_sensitive { "" } else { "(?i)" };
        let re = match regex::Regex::new(&format!("{}{}", flags, pattern)) {
            Ok(re) => re,
            Err(_) => {
                let results: Vec<MatchResult> = Vec::new();
                return serde_wasm_bindgen::to_value(&results).unwrap();
            }
        };

        let mut results: Vec<MatchResult> = lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let m = re.find(line)?;
                let highlights: Vec<usize> = (m.start()..m.end()).collect();
                Some(MatchResult {
                    line_index: idx,
                    score: 100,
                    highlights,
                })
            })
            .collect();

        results.truncate(limit);
        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    /// Perform plain text search on lines
    pub fn search_plain(
        &mut self,
        query: &str,
        lines: &JsValue,
        case_sensitive: bool,
        limit: usize,
    ) -> JsValue {
        let lines: Vec<String> = serde_wasm_bindgen::from_value(lines.clone()).unwrap_or_default();

        if query.is_empty() {
            let results: Vec<MatchResult> = Vec::new();
            return serde_wasm_bindgen::to_value(&results).unwrap();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<MatchResult> = lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let pos = if case_sensitive {
                    line.find(query)
                } else {
                    line.to_lowercase().find(&query_lower)
                }?;

                let highlights: Vec<usize> = (pos..pos + query.len()).collect();
                Some(MatchResult {
                    line_index: idx,
                    score: 100,
                    highlights,
                })
            })
            .collect();

        results.truncate(limit);
        serde_wasm_bindgen::to_value(&results).unwrap()
    }
}
