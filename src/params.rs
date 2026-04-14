use std::path::Path;

pub struct Params {
    pub file_path: String,
    pub extension: String,
    pub command: String,
}

impl Params {
    pub fn new(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 2 || args[1].is_empty() {
            return Err(
                "Not enough arguments provided. Please provide the file path as an argument.",
            );
        }

        let file_path: String = args[1].clone();
        let path = Path::new(&file_path);
        let mut extension: String = "".to_string();
        let mut command: String = "".to_string();

        if path.exists() && path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if ext_str.eq_ignore_ascii_case("xml") {
                        extension = ext_str.to_string();
                    } else if ext_str.eq_ignore_ascii_case("json") {
                        extension = ext_str.to_string();
                    } else {
                        return Err("Unsupported file type. Please provide XML or JSON file.");
                    }
                } else {
                    return Err("Failed to read the file extension. Please provide a valid file.");
                }
            } else {
                return Err(
                    "Provided file path does not exist or is not a file. Please provide a valid file path.",
                );
            }
        }

        if args.len() == 3 && !args[2].is_empty() {
            command = args[2].clone();
        }

        Ok(Params {
            file_path,
            extension,
            command,
        })
    }
}
