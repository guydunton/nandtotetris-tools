mod ast;
mod parser;
mod translate_ast;

use clap::{Arg, Command, ValueHint};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use translate_ast::translate_ast;

fn main() {
    let matches = Command::new("VM Translator")
        .about("Translate VM code to Hack assembly")
        .arg(
            Arg::new("INPUT")
                .index(1)
                .required(true)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .help("A VM language file"),
        )
        .arg_required_else_help(true)
        .get_matches();

    let path = matches
        .get_one::<String>("INPUT")
        .expect("User to provide an input path");

    // Load the assembly
    match parse_and_convert_vm(path) {
        Ok(_) => println!(),
        Err(err) => {
            println!("Failed to convert file {:?}", err);
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
enum ErrorType {
    FileError(io::Error),
    ParsingError(String),
    TranslationError(String),
    InvalidFileName,
}

fn parse_and_convert_vm(path: &str) -> Result<(), ErrorType> {
    let contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
    let statements = parser::parser(&contents).map_err(ErrorType::ParsingError)?;

    let file_name = Path::new(path)
        .file_name()
        .ok_or(ErrorType::InvalidFileName)?
        .to_owned()
        .into_string()
        .map_err(|_| ErrorType::InvalidFileName)?;

    let asm = translate_ast(statements, &file_name).map_err(ErrorType::TranslationError)?;

    // Get the hack filename
    let output_file_name = Path::new(path)
        .file_stem()
        .ok_or(ErrorType::InvalidFileName)?
        .to_owned();
    let mut out_file = PathBuf::from(path);
    out_file.set_file_name(output_file_name);
    out_file.set_extension("asm");

    // Write into a file
    fs::write(out_file, asm).map_err(ErrorType::FileError)?;

    Ok(())
}
