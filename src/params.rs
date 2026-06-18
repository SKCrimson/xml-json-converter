use std::path::Path;

pub struct Params {
    pub file_path: String,
    pub extension: String,
    pub pretty: bool,
    pub output_name: Option<String>,
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
        let extension: String;
        let mut pretty = false;
        let mut output_name: Option<String> = None;

        if !path.exists() || !path.is_file() {
            return Err("File not found. Please provide a valid file path.");
        }

        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if ext_str.eq_ignore_ascii_case("xml") || ext_str.eq_ignore_ascii_case("json") {
                    extension = ext_str.to_ascii_lowercase();
                } else {
                    return Err("Unsupported file type. Please provide XML or JSON file.");
                }
            } else {
                return Err("Failed to read the file extension. Please provide a valid file.");
            }
        } else {
            return Err("File has no extension. Please provide an XML or JSON file.");
        }

        let mut i = 2;
        while i < args.len() {
            match args[i].to_lowercase().as_str() {
                "--pretty" | "-p" => {
                    pretty = true;
                    i += 1;
                }
                "--name" | "-n" => {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        output_name = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err("Missing value for -n/--name option.");
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(Params {
            file_path,
            extension,
            pretty,
            output_name,
        })
    }
}
