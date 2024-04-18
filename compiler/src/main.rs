mod ast;
mod parser;

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, Command, ValueHint};
use parser::{parse_jack, FileInput};

enum ErrorType {
    FileError(std::io::Error),
    CompileError(String),
    SerdeError,
    FileExtensionError,
}

fn main() {
    let matches = Command::new("Jack Compiler")
        .about("A compiler for the Jack programming language")
        .arg(
            Arg::new("SOURCE")
                .index(1)
                .required(true)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .help("A Jack source file"),
        )
        .get_matches();

    // Get the file
    let path = matches
        .get_one::<String>("SOURCE")
        .expect("User to provide a source file");

    match process_source(path) {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            match err {
                ErrorType::FileError(file_err) => println!("Failed with file error: {}", file_err),
                ErrorType::CompileError(err) => println!("{}", err),
                ErrorType::SerdeError => println!("An unknown serde json error occurred"),
                ErrorType::FileExtensionError => {
                    println!("Error getting file extension within directory")
                }
            };
            std::process::exit(1);
        }
    }
}

fn process_source(path_str: &str) -> Result<(), ErrorType> {
    let path = Path::new(path_str);

    if path.is_dir() {
        let mut jack_files = Vec::new();
        for file in path.read_dir().unwrap() {
            let file_path = file.unwrap().path();
            if file_path.is_dir() {
                continue;
            }
            if file_path.extension().ok_or(ErrorType::FileExtensionError)? == "jack" {
                jack_files.push(file_path.to_str().unwrap().to_owned());
            }
        }

        for jack_file in jack_files {
            load_and_process_source(&jack_file)?;
        }
        return Ok(());
    } else {
        return load_and_process_source(path_str);
    }
}

fn load_and_process_source(path_str: &str) -> Result<(), ErrorType> {
    let path = Path::new(path_str);
    let file_contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
    let file_name = path.file_name().to_owned().unwrap().to_str().unwrap();

    let result = parse_jack(vec![FileInput::new(file_name, &file_contents)])
        .map_err(|s| ErrorType::CompileError(s))?;

    for (compiled_output, filename) in result {
        let compiled_json =
            serde_json::to_string_pretty(&compiled_output).map_err(|_| ErrorType::SerdeError)?;

        let mut original_file_path = PathBuf::from(&filename);
        original_file_path.set_extension("json");
        let output_file_name = PathBuf::from(path);
        let output_file = output_file_name.parent().unwrap().join(original_file_path);
        fs::write(output_file, compiled_json).map_err(ErrorType::FileError)?;
    }

    Ok(())
}
