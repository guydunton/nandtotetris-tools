mod ast;
mod compiler;
mod parser;
mod symbol_table;

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command, ValueHint};
use compiler::CompilationError;
use parser::{parse_jack, FileInput};

enum ErrorType {
    FileError(std::io::Error),
    ParsingError(String),
    SerdeError,
    FileExtensionError,
    CompilationError(CompilationError),
}

fn main() {
    let matches = Command::new("Jack Compiler")
        .about("A compiler for the Jack programming language")
        .arg(
            Arg::new("ast_output")
                .required(false)
                .action(ArgAction::SetTrue)
                .long("ast_output")
                .num_args(0)
                .help("Output JSON version of the AST instead of .vm files"),
        )
        .arg(
            Arg::new("SOURCE")
                .required(true)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .help("A Jack source file or directory"),
        )
        .get_matches();

    // Get the file
    let path = matches
        .get_one::<String>("SOURCE")
        .expect("User to provide a source file");

    let output_json = matches.get_flag("ast_output");

    match process_source(path, output_json) {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            match err {
                ErrorType::FileError(file_err) => println!("Failed with file error: {}", file_err),
                ErrorType::ParsingError(err) => println!("{}", err),
                ErrorType::SerdeError => println!("An unknown serde json error occurred"),
                ErrorType::FileExtensionError => {
                    println!("Error getting file extension within directory")
                }
                ErrorType::CompilationError(err) => {
                    println!("An error occurred during VM compilation: {:?}", err)
                }
            };
            std::process::exit(1);
        }
    }
}

fn process_source(path_str: &str, output_json: bool) -> Result<(), ErrorType> {
    let jack_files = find_jack_files(path_str)?;

    let source_dir = get_source_dir(path_str)?;

    process_sources(&jack_files, source_dir, output_json)?;
    Ok(())
}

fn process_sources(
    path_str: &Vec<String>,
    source_dir: &Path,
    output_json: bool,
) -> Result<(), ErrorType> {
    let mut file_names = Vec::with_capacity(path_str.len());
    for single_file in path_str {
        let path = Path::new(single_file);
        let contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
        let filename = path.file_name().to_owned().unwrap().to_str().unwrap();
        file_names.push(FileInput::new(filename, &contents));
    }

    let result = parse_jack(file_names).map_err(|s| ErrorType::ParsingError(s))?;

    // Print the json AST output
    if output_json {
        for single_file in &result.classes {
            let compiled_json = serde_json::to_string_pretty(&single_file.class)
                .map_err(|_| ErrorType::SerdeError)?;

            let mut original_file_path = PathBuf::from(&single_file.source_filename);
            original_file_path.set_extension("json");
            let output_file_name = PathBuf::from(source_dir);
            let output_file = output_file_name.join(original_file_path);
            fs::write(output_file, compiled_json).map_err(ErrorType::FileError)?;
        }
    }

    // Compile to VM commands
    let vm_output = compiler::translate_ast(&result).map_err(ErrorType::CompilationError)?;

    for vm_file in &vm_output {
        let bytecode = vm_file.vm_code.join("\n");

        let mut original_file_path = PathBuf::from(&vm_file.source_filename);
        original_file_path.set_extension("vm");
        let output_file_name = PathBuf::from(source_dir);
        let output_file = output_file_name.join(original_file_path);
        fs::write(output_file, bytecode).map_err(ErrorType::FileError)?;
    }

    Ok(())
}

fn find_jack_files(path_str: &str) -> Result<Vec<String>, ErrorType> {
    let path = Path::new(path_str);
    let mut jack_files = Vec::new();
    if path.is_dir() {
        for file in path.read_dir().unwrap() {
            let file_path = file.unwrap().path();
            if file_path.is_dir() {
                continue;
            }
            if file_path.extension().ok_or(ErrorType::FileExtensionError)? == "jack" {
                jack_files.push(file_path.to_str().unwrap().to_owned());
            }
        }
    } else {
        jack_files.push(path_str.to_owned());
    }

    Ok(jack_files)
}

fn get_source_dir(path_str: &str) -> Result<&Path, ErrorType> {
    let path = Path::new(path_str);
    let source_dir = if path.is_dir() {
        path
    } else {
        path.parent().ok_or(ErrorType::FileExtensionError)?
    };

    Ok(source_dir)
}
