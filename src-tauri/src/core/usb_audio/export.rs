use std::fs;
use std::io::Write;

pub fn save_wav(pcm: &[u8], path: &str, sample_rate: u32, num_channels: u16, bit_depth: u16) -> Result<(), String> {
    let file = fs::File::create(path).map_err(|e| format!("创建文件失败: {}", e))?;
    let mut w = std::io::BufWriter::new(file);

    let data_size = pcm.len() as u32;
    let byte_rate = sample_rate * num_channels as u32 * (bit_depth / 8) as u32;
    let block_align = num_channels * (bit_depth / 8);

    w.write_all(b"RIFF").map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&(36 + data_size).to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(b"WAVE").map_err(|e| format!("写 WAV 失败: {}", e))?;

    w.write_all(b"fmt ").map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&16u32.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&1u16.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&num_channels.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&sample_rate.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&byte_rate.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&block_align.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&bit_depth.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;

    w.write_all(b"data").map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(&data_size.to_le_bytes()).map_err(|e| format!("写 WAV 失败: {}", e))?;
    w.write_all(pcm).map_err(|e| format!("写 WAV 失败: {}", e))?;

    w.flush().map_err(|e| format!("写 WAV 失败: {}", e))?;
    Ok(())
}

pub fn save_raw(data: &[u8], path: &str) -> Result<(), String> {
    fs::write(path, data).map_err(|e| format!("保存 RAW 失败: {}", e))
}
