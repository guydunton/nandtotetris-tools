mod convert_labels;
mod convert_variables;
mod interpreter;
mod parser;
mod symbol_table;

use clap::{Arg, Command, ValueHint};
use convert_labels::{find_labels, remove_all_labels};
use convert_variables::find_variables;
use interpreter::interpret_ast;
use std::path::{Path, PathBuf};
use std::{fs, io};
use symbol_table::create_symbol_table;

use crate::parser::parse_hack;

fn main() {
    let matches = Command::new("Hack Assembler")
        .about("Compile hack assembly files into machine code")
        .arg(
            Arg::new("INPUT")
                .index(1)
                .required(true)
                .value_name("FILE")
                .value_hint(ValueHint::FilePath)
                .help("A Hack assembly file"),
        )
        .arg_required_else_help(true)
        .get_matches();

    let path = matches
        .get_one::<String>("INPUT")
        .expect("User to provide an input path");

    // Load the assembly
    match parse_and_convert_file(path) {
        Ok(_) => println!(),
        Err(err) => {
            println!("Failed to parse file with error {:?}", err);
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
enum ErrorType {
    FileError(io::Error),
    ParsingError(String),
    InvalidFileName,
}

fn parse_and_convert_file(path: &str) -> Result<(), ErrorType> {
    let contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
    let mut statements = parse_hack(&contents).map_err(ErrorType::ParsingError)?;

    // Manipulate AST

    // Create a symbol table
    let mut symbol_table = create_symbol_table();

    // Find all the labels (& their expected addresses)
    find_labels(&statements, &mut symbol_table);

    // Remove all the labels
    statements = remove_all_labels(statements);

    // Find all the variables
    find_variables(&statements, &mut symbol_table);

    // Convert to binary
    let binary = interpret_ast(&statements, &symbol_table);
    let binary_data = binary
        .into_iter()
        .map(|data| format!("{:016b}", data))
        .collect::<Vec<String>>()
        .join("\n");

    // Get the hack filename
    let output_file_name = Path::new(path)
        .file_stem()
        .ok_or(ErrorType::InvalidFileName)?
        .to_owned();
    let mut out_file = PathBuf::from(path);
    out_file.set_file_name(output_file_name);
    out_file.set_extension("hack");

    // Write into a file
    fs::write(out_file, binary_data).map_err(ErrorType::FileError)?;

    Ok(())
}
