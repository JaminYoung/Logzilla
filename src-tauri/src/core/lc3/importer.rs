use std::fs;
use std::path::Path;

pub fn import_lc3_file(path: &str) -> Result<Vec<u8>, String> {
    let p = Path::new(path);
    let ext = p.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "bin" => read_bin_file(path),
        "txt" | "csv" => {
            let first_line = read_first_line(path)?;
            if first_line.starts_with("Time") || first_line.starts_with("TIME") {
                parse_kingst_txt(path)
            } else {
                hex_text_to_binary(path)
            }
        }
        _ => {
            let first_line = read_first_line(path).unwrap_or_default();
            if first_line.starts_with("Time") || first_line.starts_with("TIME") {
                parse_kingst_txt(path)
            } else {
                hex_text_to_binary(path)
            }
        }
    }
}

fn read_bin_file(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("读取文件失败: {}", e))
}

fn read_first_line(path: &str) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(content.lines().next().unwrap_or("").to_string())
}

fn hex_text_to_binary(path: &str) -> Result<Vec<u8>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let hex_str: String = text.split_whitespace().collect();
    if hex_str.is_empty() {
        return Err("文件为空或格式不正确".to_string());
    }

    hex_to_bytes(&hex_str)
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度不是偶数".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| format!("无效的十六进制字符: {}", &hex[i..i + 2]))
        })
        .collect()
}

fn parse_kingst_txt(path: &str) -> Result<Vec<u8>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let mut raw = Vec::new();
    for line in content.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let hex_str = parts[2].trim();
            let hex_str = hex_str
                .strip_prefix("0x")
                .or_else(|| hex_str.strip_prefix("0X"))
                .unwrap_or(hex_str);
            if !hex_str.is_empty() {
                if let Ok(val) = u8::from_str_radix(hex_str, 16) {
                    raw.push(val);
                }
            }
        }
    }

    if raw.is_empty() {
        return Err("未提取到有效数据，请确认文件为Kingst逻辑分析仪导出格式".to_string());
    }

    Ok(raw)
}
