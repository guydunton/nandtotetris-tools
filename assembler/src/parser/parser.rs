use nom::branch::alt;

use super::c_statement::parse_c_statement;
use super::parse_utils::{parse_comment, parse_empty_lines};
use super::Stmt;
use super::{a_statement::parse_a_instruction, label::parse_label};

pub fn parse_hack(i: &str) -> Result<Vec<(String, Stmt)>, String> {
    // Split into lines
    let lines = i.lines();
    let mut statements = Vec::new();
    for line in lines {
        let (_, parsed_statement) = alt((
            parse_comment,
            parse_empty_lines,
            parse_label,
            parse_a_instruction,
            parse_c_statement,
        ))(line)
        .map_err(|err| format!("Found error {} on line {}", err.to_string(), line))?;

        statements.push((line.to_owned(), parsed_statement));
    }

    Ok(statements)
}
