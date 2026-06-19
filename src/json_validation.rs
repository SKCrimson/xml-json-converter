use crate::utils;

pub fn get_content(file_path: &str) -> Result<String, String> {
    utils::read_file(file_path)
}
