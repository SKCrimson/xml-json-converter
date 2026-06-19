use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod json_to_xml;
mod json_validation;
mod params;
mod utils;
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
            process::exit(1);
        }
    };

    match params.extension.as_str() {
        "xml" => {
            get_json(&params.file_path, params.pretty, params.output_name.as_deref());
        }
        "json" => {
            get_xml(&params.file_path, params.pretty, params.output_name.as_deref());
        }
        _ => {
            eprintln!("Unsupported file type. Please provide XML or JSON file.");
            process::exit(1);
        }
    }
}

fn get_json(file_path: &str, pretty: bool, output_name: Option<&str>) {
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

    let save_file_path = build_output_path(file_path, "json", output_name);

    if let Err(e) = fs::write(&save_file_path, &json_content) {
        eprintln!("Error writing JSON file: {}", e);
        process::exit(1);
    }

    println!("Successfully saved to: {}", save_file_path);
}

fn get_xml(file_path: &str, pretty: bool, output_name: Option<&str>) {
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

    let save_file_path = build_output_path(file_path, "xml", output_name);

    if let Err(e) = fs::write(&save_file_path, &xml_content) {
        eprintln!("Error writing XML file: {}", e);
        process::exit(1);
    }

    println!("Successfully saved to: {}", save_file_path);
}

fn build_output_path(input: &str, ext: &str, name: Option<&str>) -> String {
    match name {
        Some(n) => {
            let dir = Path::new(input).parent().unwrap_or(Path::new("."));
            dir.join(format!("{}.{}", n, ext)).to_string_lossy().into_owned()
        }
        None => {
            let stem = Path::new(input).with_extension("").to_string_lossy().into_owned();
            format!("{}-result.{}", stem, ext)
        }
    }
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
    println!("    -p, --pretty        Format output with indentation and line breaks.");
    println!("    -n, --name <NAME>   Custom output file name (without extension).");
    println!("    -h, --help          Print this help message.");
    println!();
    println!("OUTPUT:");
    println!("    Default: result is written next to the source file:");
    println!("      input.xml   ->  input-result.json");
    println!("      input.json  ->  input-result.xml");
    println!("    With -n myfile:");
    println!("      input.xml   ->  myfile.json");
    println!("      input.json  ->  myfile.xml");
    println!();
    println!("EXAMPLES:");
    println!("    {} C:\\data\\example.xml", exe_name);
    println!("    {} C:\\data\\example.xml -n output", exe_name);
    println!("    {} C:\\data\\example.json --pretty --name output", exe_name);
    println!();
    println!("NOTES:");
    println!("    XML -> JSON: attributes are prefixed with '@', text nodes with '#text'.");
    println!("    JSON -> XML: the root object becomes the <root> element.");
    println!();
    println!("ROUND-TRIP LIMITATIONS:");
    println!("    Conversion is one-way lossless within each direction, but a full");
    println!("    round-trip (XML->JSON->XML or JSON->XML->JSON) is NOT idempotent.");
    println!();
    println!("    XML -> JSON -> XML:");
    println!("      Attributes become JSON keys (\"@attr\") and are converted back as");
    println!("      child elements, not attributes:");
    println!("        <item id=\"1\">text</item>");
    println!("        -> {{ \"@id\": \"1\", \"#text\": \"text\" }}");
    println!("        -> <item><_id>1</_id><__text>text</__text></item>");
    println!();
    println!("    JSON -> XML -> JSON:");
    println!("      The root XML element (always named <root>) adds an extra nesting");
    println!("      level on the way back:");
    println!("        {{ \"name\": \"Alice\" }}");
    println!("        -> <root><name>Alice</name></root>");
    println!("        -> {{ \"root\": {{ \"name\": \"Alice\" }} }}");
}
