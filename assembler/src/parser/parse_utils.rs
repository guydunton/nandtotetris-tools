use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric1, line_ending, multispace0, not_line_ending, space0,
    },
    combinator::{all_consuming, map, recognize, value},
    multi::many0_count,
    sequence::{pair, tuple},
    IResult, Parser,
};

use super::Stmt;

pub fn parse_comment(i: &str) -> IResult<&str, Option<Stmt>> {
    value(None, tuple((space0, tag("//"), not_line_ending))).parse(i)
}

pub fn parse_empty_lines(i: &str) -> IResult<&str, Option<Stmt>> {
    map(all_consuming(alt((multispace0, line_ending))), |_| None)(i)
}

pub fn parse_name(i: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("."), tag("$")))),
    ))
    .parse(i)
}

#[test]
fn test_parse_comment() {
    assert!(parse_comment("// something something").is_ok());
    assert!(parse_comment("// something something\n").is_ok());
    assert!(parse_comment("    // something something\n").is_ok());
}

#[test]
fn test_parse_multilines() {
    assert!(parse_empty_lines("@21").is_err());
    assert!(parse_empty_lines("\n").is_ok());
    assert!(parse_empty_lines("").is_ok());

    assert!(parse_empty_lines("  @21").is_err());
}

#[test]
fn test_parse_name() {
    assert_eq!(parse_name("THIS_NAME").unwrap(), ("", "THIS_NAME"));
    assert_eq!(parse_name("THIS_NAME.new").unwrap(), ("", "THIS_NAME.new"));
    assert_eq!(
        parse_name("THIS_NAME.new$if_lt_4").unwrap(),
        ("", "THIS_NAME.new$if_lt_4")
    );
}
