use std::env;
use std::process;

mod json_to_xml;
mod json_validation;
mod params;
mod xml_to_json;
mod xml_validation;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments: {:?}", args);
    //* Only for testing */
    // let args: Vec<String> = vec![
    //     "target\\debug\\xml-json-converter.exe".to_string(),
    //     "C:\\Users\\krivo\\source\\rust\\xml-json-converter\\target\\no-valid.xml".to_string(),
    // ];

    let params = match params::Params::new(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    println!("File Path: {}", params.file_path);
    println!("File Extension: {}", params.extension);

    match params.extension.as_str() {
        "xml" => {
            let xml_content = match xml_validation::get_content(&params.file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error validating XML: {}", e);
                    process::exit(1);
                }
            };

            println!("XML Content: {}", xml_content);
        }
        "json" => {
            let json_content = match json_validation::get_content(&params.file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error validating JSON: {}", e);
                    process::exit(1);
                }
            };

            println!("JSON Content: {}", json_content);
        }
        _ => {
            eprintln!("Unsupported file type. Please provide XML or JSON file.");
            process::exit(1);
        }
    }
}
