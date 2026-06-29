use crate::core::search::engine::SearchEngine;
use crate::core::search::types::*;
use std::sync::Arc;
use tauri::State;

pub struct SearchState {
    pub engine: Arc<SearchEngine>,
}

#[tauri::command]
pub fn search_logs(
    query: String,
    panel: usize,
    mode: Option<String>,
    case_sensitive: Option<bool>,
    limit: Option<usize>,
    state: State<crate::AppState>,
    search_state: State<SearchState>,
) -> Result<SearchResult, String> {
    if query.is_empty() {
        return Ok(SearchResult {
            matches: vec![],
            total_count: 0,
            query,
            elapsed_ms: 0.0,
        });
    }

    let logs = if panel == 0 {
        state.logs.lock().unwrap()
    } else {
        state.logs2.lock().unwrap()
    };

    let search_mode = match mode.as_deref() {
        Some("regex") => SearchMode::Regex,
        Some("plain") => SearchMode::Plain,
        _ => SearchMode::Fuzzy,
    };

    let case_mode = if case_sensitive.unwrap_or(false) {
        CaseMode::Sensitive
    } else {
        CaseMode::Smart
    };

    let result = search_state.engine.search(
        &query,
        &logs,
        search_mode,
        case_mode,
        limit.unwrap_or(100),
    );

    Ok(result)
}

#[tauri::command]
pub fn get_logs(
    panel: usize,
    since_version: Option<u64>,
    known_lines: Option<usize>,
    state: State<crate::AppState>,
) -> Result<(Vec<String>, u64), String> {
    let (logs, version) = if panel == 0 {
        (state.logs.lock().map_err(|e| e.to_string())?,
         state.logs_version.lock().map_err(|e| e.to_string())?)
    } else {
        (state.logs2.lock().map_err(|e| e.to_string())?,
         state.logs2_version.lock().map_err(|e| e.to_string())?)
    };

    // 如果指定了版本号，且当前版本号未变化，返回空
    if let Some(since) = since_version {
        if *version <= since {
            return Ok((vec![], *version));
        }
    }

    // base = 已丢弃的行数；前端传来的 known_lines 是累计行号。
    // 用 known_lines - base 得到缓冲区内的起始下标；若前端落后于已丢弃位置，则从头返回。
    let base = if panel == 0 {
        *state.logs_base.lock().map_err(|e| e.to_string())?
    } else {
        *state.logs2_base.lock().map_err(|e| e.to_string())?
    };
    let known = known_lines.unwrap_or(0);
    let start = known.saturating_sub(base).min(logs.len());
    let new_lines = logs[start..].to_vec();

    Ok((new_lines, *version))
}

#[tauri::command]
pub fn update_logs(
    logs: Vec<String>,
    panel: usize,
    state: State<crate::AppState>,
) -> Result<(), String> {
    if panel == 0 {
        *state.logs.lock().map_err(|e| e.to_string())? = logs;
        *state.logs_version.lock().map_err(|e| e.to_string())? += 1;
    } else {
        *state.logs2.lock().map_err(|e| e.to_string())? = logs;
        *state.logs2_version.lock().map_err(|e| e.to_string())? += 1;
    }
    Ok(())
}

#[tauri::command]
pub fn clear_logs(
    panel: usize,
    state: State<crate::AppState>,
) -> Result<(), String> {
    if panel == 0 {
        state.logs.lock().map_err(|e| e.to_string())?.clear();
        *state.logs_base.lock().map_err(|e| e.to_string())? = 0;
        *state.logs_version.lock().map_err(|e| e.to_string())? += 1;
    } else {
        state.logs2.lock().map_err(|e| e.to_string())?.clear();
        *state.logs2_base.lock().map_err(|e| e.to_string())? = 0;
        *state.logs2_version.lock().map_err(|e| e.to_string())? += 1;
    }
    Ok(())
}
