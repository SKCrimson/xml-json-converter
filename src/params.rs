use std::path::Path;

pub struct Params {
    pub file_path: String,
    pub extension: String,
    pub indent: usize,
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

        if !path.exists() || !path.is_file() {
            return Err("File not found. Please provide a valid file path.".into());
        }

        let mut indent: usize = 0;
        let mut output_name: Option<String> = None;
        let mut from_format: Option<String> = None;

        let mut i = 2;
        while i < args.len() {
            match args[i].to_lowercase().as_str() {
                "--pretty" | "-p" => {
                    if indent == 0 { indent = 4; }
                    i += 1;
                }
                "--indent" | "-i" => {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        indent = args[i + 1].parse::<usize>().map_err(|_| {
                            format!("Invalid value for --indent: '{}' (expected a positive integer)", args[i + 1])
                        })?;
                        i += 2;
                    } else {
                        return Err("Missing value for --indent option.".into());
                    }
                }
                "--name" | "-n" => {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        output_name = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err("Missing value for -n/--name option.".into());
                    }
                }
                "--from" => {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        let fmt = args[i + 1].to_lowercase();
                        if fmt != "xml" && fmt != "json" {
                            return Err(format!("Invalid value for --from: '{}' (expected 'xml' or 'json')", args[i + 1]));
                        }
                        from_format = Some(fmt);
                        i += 2;
                    } else {
                        return Err("Missing value for --from option.".into());
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

        let extension = if let Some(fmt) = from_format {
            fmt
        } else {
            match path.extension().and_then(|e| e.to_str()) {
                Some(e) if e.eq_ignore_ascii_case("xml") || e.eq_ignore_ascii_case("json") => {
                    e.to_ascii_lowercase()
                }
                Some(_) => return Err("Unsupported file type. Please provide XML or JSON file.".into()),
                None => return Err("File has no extension. Use --from xml|json to specify the format explicitly.".into()),
            }
        };

        Ok(Params {
            file_path,
            extension,
            indent,
            output_name,
        })
    }
}
