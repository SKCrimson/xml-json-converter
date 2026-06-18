use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod json_to_xml;
mod json_validation;
mod params;
mod xml_to_json;
mod xml_validation;

fn main() {
    let args: Vec<String> = env::args().collect();

    let exe_name = Path::new(&args[0])
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("xml-json-converter");

    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help(exe_name);
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

fn print_help(exe_name: &str) {
    println!("XML-JSON Converter");
    println!("Converts between XML and JSON formats.");
    println!();
    println!("USAGE:");
    println!("    {} <FILE_PATH> [OPTIONS]", exe_name);
    println!();
    println!("ARGUMENTS:");
    println!("    <FILE_PATH>    Path to the source file (.xml or .json).");
    println!("                   Format is detected automatically from the extension.");
    println!();
    println!("OPTIONS:");
    println!("    -p, --pretty   Format output with indentation and line breaks.");
    println!("    -h, --help     Print this help message.");
    println!();
    println!("OUTPUT:");
    println!("    The result is written next to the source file:");
    println!("      input.xml   ->  input-result.json");
    println!("      input.json  ->  input-result.xml");
    println!();
    println!("EXAMPLES:");
    println!("    {} C:\\data\\example.xml", exe_name);
    println!("    {} C:\\data\\example.json --pretty", exe_name);
    println!("    {} C:\\data\\example.json -p", exe_name);
    println!();
    println!("NOTES:");
    println!("    XML -> JSON: attributes are prefixed with '@', text nodes with '#text'.");
    println!("    JSON -> XML: the root object becomes the <root> element.");
}
