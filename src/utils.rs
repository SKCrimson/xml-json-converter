use std::fs;

pub fn read_file(file_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read the file: {}", e))?;

    if content.is_empty() {
        return Err("Content is empty".to_string());
    }

    Ok(content)
}
