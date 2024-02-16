use nom::bytes::complete::tag;
use nom::character::complete::{char, space0};
use nom::combinator::{all_consuming, map, opt};
use nom::sequence::tuple;
use nom::{branch::alt, IResult};

use crate::parser::{Command, Dest, Operation};

use super::parse_utils::parse_comment;
use super::{Jump, Stmt};

fn parse_destination(i: &str) -> IResult<&str, Dest> {
    // The order of these is important
    map(
        alt((
            tag("AMD"),
            tag("MD"),
            tag("AM"),
            tag("AD"),
            tag("A"),
            tag("M"),
            tag("D"),
            tag("0"),
        )),
        |character| match character {
            "AMD" => Dest::AMD,
            "MD" => Dest::MD,
            "AM" => Dest::AM,
            "AD" => Dest::AD,
            "M" => Dest::M,
            "D" => Dest::D,
            "A" => Dest::A,
            "0" => Dest::NULL,
            _ => panic!("Destination parser failed with character {}", character),
        },
    )(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    map(
        alt((
            alt((
                tag("0"),
                tag("1"),
                tag("-1"),
                tag("!D"),
                tag("!A"),
                tag("!M"),
                tag("-D"),
                tag("-A"),
                tag("-M"),
                tag("D+1"),
                tag("A+1"),
                tag("M+1"),
                tag("D-1"),
                tag("A-1"),
                tag("M-1"),
            )),
            alt((
                tag("D+A"),
                tag("A+D"),
                tag("D+M"),
                tag("M+D"),
                tag("D-A"),
                tag("D-M"),
                tag("A-D"),
                tag("M-D"),
                tag("D&A"),
                tag("D&M"),
                tag("D|A"),
                tag("D|M"),
                tag("D"),
                tag("A"),
                tag("M"),
            )),
        )),
        |operation_text| match operation_text {
            "0" => Operation::Zero,
            "1" => Operation::One,
            "-1" => Operation::MinusOne,
            "!D" => Operation::NotD,
            "!A" => Operation::NotA,
            "!M" => Operation::NotM,
            "-D" => Operation::MinusD,
            "-A" => Operation::MinusA,
            "-M" => Operation::MinusM,
            "D+1" => Operation::DPlus1,
            "A+1" => Operation::APlus1,
            "M+1" => Operation::MPlus1,
            "D-1" => Operation::DMinus1,
            "A-1" => Operation::AMinus1,
            "M-1" => Operation::MMinus1,
            "D+A" => Operation::DPlusA,
            "A+D" => Operation::DPlusA,
            "D+M" => Operation::DPlusM,
            "M+D" => Operation::DPlusM,
            "D-A" => Operation::DMinusA,
            "D-M" => Operation::DMinusM,
            "A-D" => Operation::AMinusD,
            "M-D" => Operation::MMinusD,
            "D&A" => Operation::DAndA,
            "D&M" => Operation::DAndM,
            "D|A" => Operation::DOrA,
            "D|M" => Operation::DOrM,
            "D" => Operation::D,
            "A" => Operation::A,
            "M" => Operation::M,
            _ => panic!("Missing operation with text {}", operation_text),
        },
    )(i)
}

fn parse_jump(i: &str) -> IResult<&str, Jump> {
    map(
        alt((
            tag("JGT"),
            tag("JEQ"),
            tag("JGE"),
            tag("JLT"),
            tag("JNE"),
            tag("JLE"),
            tag("JMP"),
        )),
        |jump_text| match jump_text {
            "JGT" => Jump::JGT,
            "JEQ" => Jump::JEQ,
            "JGE" => Jump::JGE,
            "JLT" => Jump::JLT,
            "JNE" => Jump::JNE,
            "JLE" => Jump::JLE,
            "JMP" => Jump::JMP,
            _ => panic!("Failed to handle jump {}", jump_text),
        },
    )(i)
}

pub fn parse_c_statement(i: &str) -> IResult<&str, Option<Stmt>> {
    all_consuming(alt((
        map(
            tuple((
                space0,
                parse_destination,
                char('='),
                parse_operation,
                opt(parse_comment),
            )),
            |(_, dest, _, operation, _)| {
                Some(Stmt::C(Command {
                    dest: Some(dest),
                    operation: operation,
                    jump: None,
                }))
            },
        ),
        map(
            tuple((
                space0,
                parse_operation,
                char(';'),
                space0,
                parse_jump,
                opt(parse_comment),
            )),
            |(_, operation, _, _, jump, _)| {
                Some(Stmt::C(Command {
                    dest: None,
                    operation: operation,
                    jump: Some(jump),
                }))
            },
        ),
    )))(i)
}

#[allow(dead_code)]
fn command(dest: Dest, operation: Operation) -> Command {
    Command {
        dest: Some(dest),
        operation: operation,
        jump: None,
    }
}

#[allow(dead_code)]
fn jump_command(operation: Operation, jump: Jump) -> Command {
    Command {
        dest: None,
        operation: operation,
        jump: Some(jump),
    }
}

#[test]
fn test_c_instruction() {
    assert_eq!(
        parse_c_statement("D=M").unwrap(),
        ("", Some(Stmt::C(command(Dest::D, Operation::M),)))
    );
    assert_eq!(
        parse_c_statement("AMD=!D").unwrap(),
        ("", Some(Stmt::C(command(Dest::AMD, Operation::NotD))))
    );
    assert_eq!(
        parse_c_statement("D=D-A").unwrap(),
        ("", Some(Stmt::C(command(Dest::D, Operation::DMinusA))))
    );
    assert_eq!(
        parse_c_statement("  D=D-A").unwrap(),
        ("", Some(Stmt::C(command(Dest::D, Operation::DMinusA))))
    );
    assert_eq!(
        parse_c_statement("D=D-A // plus a comment").unwrap(),
        ("", Some(Stmt::C(command(Dest::D, Operation::DMinusA))))
    );
    assert_eq!(
        parse_c_statement("D=A+D").unwrap(),
        ("", Some(Stmt::C(command(Dest::D, Operation::DPlusA))))
    );

    // Test a jump instruction
    assert_eq!(
        parse_c_statement("0;JMP").unwrap(),
        ("", Some(Stmt::C(jump_command(Operation::Zero, Jump::JMP))))
    );
    assert_eq!(
        parse_c_statement("D;JMP").unwrap(),
        ("", Some(Stmt::C(jump_command(Operation::D, Jump::JMP))))
    );
    assert_eq!(
        parse_c_statement("0; JMP").unwrap(),
        ("", Some(Stmt::C(jump_command(Operation::Zero, Jump::JMP))))
    );
    assert_eq!(
        parse_c_statement("    0; JMP").unwrap(),
        ("", Some(Stmt::C(jump_command(Operation::Zero, Jump::JMP))))
    );

    // Test that everything is consumed
    assert!(parse_c_statement("D=D+").is_err());
    assert!(parse_c_statement("A=A&D").is_err());
}
