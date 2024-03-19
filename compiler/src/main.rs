mod ast;
mod parser;

use std::fs;
use std::path::Path;

use clap::{Arg, Command, ValueHint};
use parser::{parse_jack, FileInput};

enum ErrorType {
    FileError(std::io::Error),
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

    match load_and_process_source(path) {
        Ok(_) => std::process::exit(0),
        Err(err) => match err {
            ErrorType::FileError(file_err) => println!("Failed with file error: {}", file_err),
        },
    }
}

fn load_and_process_source(path_str: &str) -> Result<(), ErrorType> {
    let path = Path::new(path_str);
    let file_contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
    let file_name = path.file_name().to_owned().unwrap().to_str().unwrap();

    let _ = parse_jack(vec![FileInput::new(file_name, &file_contents)]);

    Ok(())
}
