use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub matches: Vec<MatchItem>,
    pub total_count: usize,
    pub query: String,
    pub elapsed_ms: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchItem {
    pub line_index: usize,
    pub score: i64,
    pub line_text: String,
    pub highlights: Vec<HighlightRange>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HighlightRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    Regex,
    Plain,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CaseMode {
    Smart,
    Sensitive,
    Insensitive,
}
