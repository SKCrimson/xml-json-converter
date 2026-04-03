use std::fs;

pub fn get_content(file_path: &str) -> Result<String, &'static str> {
    let content = fs::read_to_string(file_path)
        .map_err(|_| "Failed to read the file. Please provide a valid JSON file.")?;

    if content.len() == 0 {
        return Err("Content is empty");
    }

    Ok(content)
}