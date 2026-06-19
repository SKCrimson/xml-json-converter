use std::path::Path;

pub struct Params {
    pub file_path: String,
    pub extension: String,
    pub pretty: bool,
    pub output_name: Option<String>,
}

impl Params {
    pub fn new(args: &[String]) -> Result<Self, String> {
        if args.len() < 2 || args[1].is_empty() {
            return Err(
                "Not enough arguments provided. Please provide the file path as an argument."
                    .into(),
            );
        }

        let file_path: String = args[1].clone();
        let path = Path::new(&file_path);
        let mut pretty = false;
        let mut output_name: Option<String> = None;

        if !path.exists() || !path.is_file() {
            return Err("File not found. Please provide a valid file path.".into());
        }

        let extension = match path.extension().and_then(|e| e.to_str()) {
            Some(e) if e.eq_ignore_ascii_case("xml") || e.eq_ignore_ascii_case("json") => {
                e.to_ascii_lowercase()
            }
            Some(_) => return Err("Unsupported file type. Please provide XML or JSON file.".into()),
            None => return Err("File has no extension. Please provide an XML or JSON file.".into()),
        };

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
                        return Err("Missing value for -n/--name option.".into());
                    }
                }
                _ => {
                    return Err(format!(
                        "Unknown option: '{}'. Use --help to see available options.",
                        args[i]
                    ));
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
