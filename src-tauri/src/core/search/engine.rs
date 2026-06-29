use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::sync::Mutex;
use std::time::Instant;

use super::types::*;

pub struct SearchEngine {
    matcher: Mutex<Matcher>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: Mutex::new(Matcher::new(Config::DEFAULT)),
        }
    }

    pub fn search(
        &self,
        query: &str,
        lines: &[String],
        mode: SearchMode,
        case_mode: CaseMode,
        limit: usize,
    ) -> SearchResult {
        let start = Instant::now();

        if query.is_empty() {
            return SearchResult {
                matches: vec![],
                total_count: 0,
                query: query.to_string(),
                elapsed_ms: 0.0,
            };
        }

        let case = match case_mode {
            CaseMode::Smart => {
                if query.chars().any(|c| c.is_uppercase()) {
                    CaseMatching::Respect
                } else {
                    CaseMatching::Ignore
                }
            }
            CaseMode::Sensitive => CaseMatching::Respect,
            CaseMode::Insensitive => CaseMatching::Ignore,
        };

        let mut matches: Vec<MatchItem> = match mode {
            SearchMode::Fuzzy => {
                let pattern = Pattern::new(query, case, Normalization::Smart, AtomKind::Fuzzy);
                let mut matcher = self.matcher.lock().unwrap();
                let mut indices_buf = Vec::new();
                let mut char_buf = Vec::new();

                lines
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, line)| {
                        indices_buf.clear();
                        char_buf.clear();
                        let haystack = Utf32Str::new(line, &mut char_buf);
                        let score = pattern.score(haystack, &mut matcher)?;
                        pattern.indices(haystack, &mut matcher, &mut indices_buf);
                        
                        let highlights = indices_buf
                            .iter()
                            .map(|&i| HighlightRange { start: i as usize, end: i as usize + 1 })
                            .collect();

                        Some(MatchItem {
                            line_index: idx,
                            score: score as i64,
                            line_text: line.clone(),
                            highlights,
                        })
                    })
                    .collect()
            }
            SearchMode::Regex => {
                let flags = if matches!(case, CaseMatching::Ignore) {
                    "(?i)"
                } else {
                    ""
                };
                let re = match regex::Regex::new(&format!("{}{}", flags, query)) {
                    Ok(re) => re,
                    Err(_) => {
                        return SearchResult {
                            matches: vec![],
                            total_count: 0,
                            query: query.to_string(),
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        }
                    }
                };

                lines
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, line)| {
                        let caps = re.captures(line)?;
                        let m = caps.get(0)?;
                        Some(MatchItem {
                            line_index: idx,
                            score: 100,
                            line_text: line.clone(),
                            highlights: vec![HighlightRange {
                                start: m.start(),
                                end: m.end(),
                            }],
                        })
                    })
                    .collect()
            }
            SearchMode::Plain => {
                let query_lower = query.to_lowercase();
                let is_ignore = matches!(case, CaseMatching::Ignore);

                lines
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, line)| {
                        let pos = if is_ignore {
                            line.to_lowercase().find(&query_lower)
                        } else {
                            line.find(query)
                        };
                        pos.map(|p| MatchItem {
                            line_index: idx,
                            score: 100,
                            line_text: line.clone(),
                            highlights: vec![HighlightRange {
                                start: p,
                                end: p + query.len(),
                            }],
                        })
                    })
                    .collect()
            }
        };

        matches.sort_by(|a, b| b.score.cmp(&a.score));

        let total_count = matches.len();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        matches.truncate(limit);

        SearchResult {
            matches,
            total_count,
            query: query.to_string(),
            elapsed_ms: elapsed,
        }
    }
}
