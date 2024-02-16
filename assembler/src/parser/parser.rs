use nom::IResult;
use nom::{branch::alt, Parser};

use super::c_statement::parse_c_statement;
use super::parse_utils::{parse_comment, parse_empty_lines};
use super::Stmt;
use super::{a_statement::parse_a_instruction, label::parse_label};

pub fn parse_hack(i: &str) -> Result<Vec<Stmt>, String> {
    // Split into lines
    let lines = i.lines();
    let statements: Vec<IResult<&str, Option<Stmt>>> = lines
        .map(|line| {
            alt((
                parse_comment,
                parse_empty_lines,
                parse_label,
                parse_a_instruction,
                parse_c_statement,
            ))
            .parse(line)
        })
        .collect();

    // Find first error
    let first_error = statements
        .iter()
        .enumerate()
        .find(|(_, result)| result.is_err())
        .map(|(i, result)| format!("Found error {} on line {}", result.as_ref().unwrap_err(), i));

    if let Some(err) = first_error {
        return Err(err);
    }

    // Remove all None values
    let final_statements: Vec<Stmt> = statements
        .into_iter()
        .map(|line| line.unwrap())
        .filter_map(|line| line.1)
        .collect();

    Ok(final_statements)
}
