use std::io::BufRead;
use tauri::State;

#[derive(serde::Serialize)]
pub struct LogWindow {
    pub start_line: usize,
    pub lines: Vec<String>,
}

#[tauri::command]
pub fn get_logs(
    panel: usize,
    since_version: Option<u64>,
    known_lines: Option<usize>,
    state: State<crate::AppState>,
) -> Result<(Vec<String>, u64, usize, usize), String> {
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
            return Ok((vec![], *version, next_known_lines, base));
        }
    }

    let known = known_lines.unwrap_or(0);
    let start = known.saturating_sub(base).min(logs.len());
    let new_lines = logs[start..].to_vec();

    Ok((new_lines, *version, next_known_lines, base))
}

#[tauri::command]
pub fn read_log_window(
    path: String,
    center_line: usize,
    before: usize,
    after: usize,
) -> Result<LogWindow, String> {
    let file = std::fs::File::open(&path).map_err(|e| format!("Failed to open log file: {}", e))?;
    let reader = std::io::BufReader::new(file);

    let start_line = center_line.saturating_sub(before);
    let end_line = center_line.saturating_add(after).saturating_add(1);
    let max_lines = before.saturating_add(after).saturating_add(1);
    let mut lines = Vec::with_capacity(max_lines.min(10_000));

    for (idx, line) in reader.lines().enumerate() {
        if idx < start_line {
            continue;
        }
        if idx >= end_line {
            break;
        }
        lines.push(line.map_err(|e| format!("Failed to read log file: {}", e))?);
    }

    Ok(LogWindow { start_line, lines })
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
