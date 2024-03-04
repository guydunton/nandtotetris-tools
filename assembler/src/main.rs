mod convert_labels;
mod convert_variables;
mod interpreter;
mod parser;
mod symbol_table;

use clap::{Arg, ArgAction, Command, ValueHint};
use convert_labels::{find_labels, remove_all_labels};
use convert_variables::find_variables;
use interpreter::interpret_ast;
use parser::Stmt;
use std::path::PathBuf;
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
        .arg(
            Arg::new("symbol")
                .short('s')
                .long("symbol")
                .action(ArgAction::SetTrue)
                .required(false)
                .help("Save a symbol file in the same directory as the output"),
        )
        .arg_required_else_help(true)
        .get_matches();

    let path = matches
        .get_one::<String>("INPUT")
        .expect("User to provide an input path");

    let generate_symbol_file = matches
        .get_one::<bool>("symbol")
        .map(|b| b.clone())
        .unwrap_or(false);

    // Load the assembly
    match parse_and_convert_file(path, generate_symbol_file) {
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
    SaveSymbolFileError(io::Error),
    ParsingError(String),
}

fn parse_and_convert_file(path: &str, generate_symbol_file: bool) -> Result<(), ErrorType> {
    let contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
    let lines = parse_hack(&contents).map_err(ErrorType::ParsingError)?;

    if generate_symbol_file {
        // Create the file path
        let mut symbol_file_path = PathBuf::from(path);
        symbol_file_path.set_extension("symbol");

        save_symbol_file(&symbol_file_path, &lines)?;
    }

    // Remove empty statements
    let mut statements = lines
        .into_iter()
        .filter(|stmt| !matches!(stmt.1, Stmt::Empty))
        .map(|(_, s)| s)
        .collect();

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
    let mut out_file = PathBuf::from(path);
    out_file.set_extension("hack");

    // Write into a file
    fs::write(out_file, binary_data).map_err(ErrorType::FileError)?;

    Ok(())
}

fn save_symbol_file(
    symbol_file_path: &PathBuf,
    statements: &Vec<(String, Stmt)>,
) -> Result<(), ErrorType> {
    let mut symbols: Vec<String> = Vec::new();
    let mut line_counter = 0;

    for (code, statement) in statements {
        match statement {
            Stmt::A(_) | Stmt::C(_) => {
                // Use the line number & increase
                symbols.push(format!("{} {}", line_counter, code));
                line_counter += 1;
            }
            _ => {
                // Print the line but don't increase line number
                symbols.push(format!("{} {}", line_counter, code));
            }
        }
    }

    // Save the symbol file
    fs::write(symbol_file_path, symbols.join("\n")).map_err(ErrorType::SaveSymbolFileError)?;

    Ok(())
}
