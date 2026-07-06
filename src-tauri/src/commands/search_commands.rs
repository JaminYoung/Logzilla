use tauri::State;

#[tauri::command]
pub fn get_logs(
    panel: usize,
    since_version: Option<u64>,
    known_lines: Option<usize>,
    state: State<crate::AppState>,
) -> Result<(Vec<String>, u64, usize), String> {
    let (logs, version) = if panel == 0 {
        (
            state.logs.lock().map_err(|e| e.to_string())?,
            state.logs_version.lock().map_err(|e| e.to_string())?,
        )
    } else {
        (
            state.logs2.lock().map_err(|e| e.to_string())?,
            state.logs2_version.lock().map_err(|e| e.to_string())?,
        )
    };

    let base = if panel == 0 {
        *state.logs_base.lock().map_err(|e| e.to_string())?
    } else {
        *state.logs2_base.lock().map_err(|e| e.to_string())?
    };
    let next_known_lines = base + logs.len();

    if let Some(since) = since_version {
        if *version <= since {
            return Ok((vec![], *version, next_known_lines));
        }
    }

    let known = known_lines.unwrap_or(0);
    let start = known.saturating_sub(base).min(logs.len());
    let new_lines = logs[start..].to_vec();

    Ok((new_lines, *version, next_known_lines))
}

#[tauri::command]
pub fn clear_logs(panel: usize, state: State<crate::AppState>) -> Result<(), String> {
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
