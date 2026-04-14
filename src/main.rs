use std::env;
use std::fs;
use std::process;
use std::io::{self, Read};

mod json_to_xml;
mod json_validation;
mod params;
mod xml_to_json;
mod xml_validation;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }

    let params = match params::Params::new(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    match params.extension.as_str() {
        "xml" => {
            get_json(&params.file_path, params.command == "pretty");
        }
        "json" => {
            get_xml(&params.file_path, params.command == "pretty");
        }
        _ => {
            eprintln!("Unsupported file type. Please provide XML or JSON file.");
            process::exit(1);
        }
    }
}

fn get_json(file_path: &str, pretty: bool) {
    let xml_content = match xml_validation::get_content(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error validating XML: {}", e);
            process::exit(1);
        }
    };

    let json_content = match xml_to_json::convert(&xml_content, pretty) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error converting XML to JSON: {}", e);
            process::exit(1);
        }
    };

    let save_file_path = format!("{}-result.json", file_path.trim_end_matches(".xml"));

    if let Err(e) = fs::write(&save_file_path, &json_content) {
        eprintln!("Error writing JSON file: {}", e);
        process::exit(1);
    }

    println!("Successfully saved to: {}", save_file_path);
}

fn get_xml(file_path: &str, pretty: bool) {
    let json_content = match json_validation::get_content(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error validating JSON: {}", e);
            process::exit(1);
        }
    };

    let xml_content = match json_to_xml::convert(&json_content, pretty) {
        Ok(xml) => xml,
        Err(e) => {
            eprintln!("Error converting JSON to XML: {}", e);
            process::exit(1);
        }
    };

    let save_file_path = format!("{}-result.xml", file_path.trim_end_matches(".json"));

    if let Err(e) = fs::write(&save_file_path, &xml_content) {
        eprintln!("Error writing XML file: {}", e);
        process::exit(1);
    }

    println!("Successfully saved to: {}", save_file_path);
}

fn print_help() {
    let help_text = r#"
XML-JSON Converter CLI
A high-performance, zero-copy tool for converting between XML and JSON formats.

USAGE:
    xml-json-converter.exe <FILE_PATH> [OPTIONS]

ARGUMENTS:
    <FILE_PATH>    Path to the source file (.xml or .json).
                   The application automatically detects the format based 
                   on the file extension.

OPTIONS:
    --pretty 
       or 
    -p             Enable human-readable output with proper indentation 
                   and line breaks. If omitted, the output will be minified.

EXAMPLES:
    1. Basic conversion (minified):
       xml-json-converter.exe C:\data\example.xml

    2. Pretty-print conversion (human-readable):
       xml-json-converter.exe C:\data\example.json --pretty
    or
       xml-json-converter.exe C:\data\example.json -p

NOTES:
    - XML to JSON: Attributes are prefixed with '@' and text nodes with '#text'.
    - JSON to XML: The root object is wrapped in a <root> tag by default.
"#;

    println!("{}", help_text);

    println!("Press any key...");
    let _ = io::stdin().read(&mut [0u8]);
}
