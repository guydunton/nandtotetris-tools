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
                .help("A VM language file or directory of files"),
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
    FileExtensionError,
}

fn parse_and_convert_vm(path: &str) -> Result<(), ErrorType> {
    let file = Path::new(path);
    if file.is_file() {
        let contents = fs::read_to_string(path).map_err(ErrorType::FileError)?;
        let statements = parser::parser(&contents).map_err(ErrorType::ParsingError)?;

        let file_name = file
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
    } else if file.is_dir() {
        /*
        if directory search directory for all *.vm files:
            dump all the contents of the VM files into the same virtual "file"
            add the following bootstrap code to the start:
                SP=256
                goto Sys.init

            Then compile the file
         */
        // Find all the .vm files
        let mut vm_files = Vec::new();
        for file in file.read_dir().unwrap() {
            let file_path = file.unwrap().path();
            if file_path.is_dir() {
                continue;
            }
            if file_path.extension().ok_or(ErrorType::FileExtensionError)? == "vm" {
                vm_files.push(file_path);
            }
        }

        /*
        Bootstrap with the code:
            SP=256
            goto Sys.init
         */
        let mut final_assembly = String::from(
            r#"@261
D=A
@SP
M=D
@Sys.init
0;JMP
"#,
        );

        for file in vm_files.iter() {
            let file_contents = fs::read_to_string(file).map_err(ErrorType::FileError)?;

            let file_name = file
                .file_name()
                .ok_or(ErrorType::InvalidFileName)?
                .to_owned()
                .into_string()
                .map_err(|_| ErrorType::InvalidFileName)?;

            let statements = parser::parser(&file_contents).map_err(ErrorType::ParsingError)?;
            let asm = translate_ast(statements, &file_name).map_err(ErrorType::TranslationError)?;

            final_assembly.push_str(&asm);
            final_assembly.push('\n');
        }

        // Get the hack filename
        let output_file_name = Path::new(path)
            .file_stem()
            .ok_or(ErrorType::InvalidFileName)?
            .to_owned()
            .into_string()
            .map_err(|_| ErrorType::InvalidFileName)?;

        let out_file = file.join(format!("{}.asm", output_file_name));

        // Write into a file
        fs::write(out_file, final_assembly).map_err(ErrorType::FileError)?;
    }
    Ok(())
}
