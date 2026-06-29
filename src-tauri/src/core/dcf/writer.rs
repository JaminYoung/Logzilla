use anyhow::Result;

pub fn save_dcf(path: &str, data: &[u8]) -> Result<()> {
    std::fs::write(path, data)?;
    Ok(())
}
