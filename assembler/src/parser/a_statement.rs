use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, u16},
    combinator::{map, opt},
    sequence::tuple,
    IResult, Parser,
};

use super::ast::{Address, Stmt};
use super::parse_utils::{parse_comment, parse_name};

pub fn parse_a_instruction(i: &str) -> IResult<&str, Stmt> {
    map(
        tuple((
            space0,
            tag("@"),
            alt((
                map(u16, |val| Address::Value(val)),
                map(parse_name, |name| Address::Symbol(name.to_string())),
            )),
            opt(parse_comment),
        )),
        |(_, _, address, _)| Stmt::A(address.clone()),
    )
    .parse(i)
}

#[test]
fn test_parse_a_instruction() {
    assert_eq!(
        parse_a_instruction("@123").unwrap().1,
        Stmt::A(Address::Value(123))
    );
    assert_eq!(
        parse_a_instruction("  @i").unwrap().1,
        Stmt::A(Address::Symbol("i".to_string()))
    );
    assert_eq!(
        parse_a_instruction("@R0").unwrap().1,
        Stmt::A(Address::Symbol("R0".to_string()))
    );
    assert_eq!(
        parse_a_instruction("@12 // Plus a comment").unwrap(),
        ("", Stmt::A(Address::Value(12)))
    );

    assert_eq!(
        parse_a_instruction("@SCREEN").unwrap(),
        ("", Stmt::A(Address::Symbol("SCREEN".to_string())))
    );
    assert_eq!(
        parse_a_instruction("@KBD").unwrap(),
        ("", Stmt::A(Address::Symbol("KBD".to_string())))
    );
}
