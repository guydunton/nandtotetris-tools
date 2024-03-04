use nom::{
    character::complete::{char, space0},
    combinator::{map, opt},
    sequence::delimited,
    IResult, Parser,
};

use super::ast::Stmt;
use super::parse_utils::{parse_comment, parse_name};

pub fn parse_label(i: &str) -> IResult<&str, Stmt> {
    let label_parser = delimited(char('('), parse_name, char(')'));

    map(
        delimited(space0, label_parser, opt(parse_comment)),
        |label| Stmt::Label(label.to_string()),
    )
    .parse(i)
}

#[test]
fn test_parse_label() {
    assert_eq!(
        parse_label("(LABEL)").unwrap(),
        ("", Stmt::Label("LABEL".to_string()))
    );
    assert_eq!(
        parse_label("(A_LABEL)").unwrap(),
        ("", Stmt::Label("A_LABEL".to_string()))
    );
    assert_eq!(
        parse_label("(A_LABEL) // Plus a comment").unwrap(),
        ("", Stmt::Label("A_LABEL".to_string()))
    );
    assert_eq!(
        parse_label("(A_LABEL)// Plus a comment").unwrap(),
        ("", Stmt::Label("A_LABEL".to_string()))
    );
}
