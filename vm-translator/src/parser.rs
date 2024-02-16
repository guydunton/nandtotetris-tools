use crate::ast::{Address, MemorySegment, Operation, Stmt};
use nom::character::complete::{line_ending, multispace0, not_line_ending, space0, space1, u32};
use nom::combinator::all_consuming;
use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::tuple, IResult};

pub fn parser(text: &str) -> Result<Vec<Stmt>, String> {
    let lines = text.lines();

    let mut statements = vec![];
    for line in lines {
        let (_, operation) = parse_operation(line)
            .map_err(|err| format!("Error occurred parsing line {}: {}", line, err))?;

        if let Some(op) = operation {
            statements.push(Stmt {
                operation: op,
                text: line.to_owned(),
            });
        }
    }

    Ok(statements)
}

fn parse_operation(i: &str) -> IResult<&str, Option<Operation>> {
    alt((
        parse_push,
        parse_pop,
        parse_binary_operations,
        parse_unary_operations,
        parse_comment,
        parse_empty_lines,
    ))(i)
}

fn parse_push(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((tag("push"), space1, parse_memory_segment, space1, u32)),
        |(_, _, memory_segment, _, address)| {
            Some(Operation::Push(Address {
                memory_segment,
                address,
            }))
        },
    )(i)
}

fn parse_pop(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((tag("pop"), space1, parse_memory_segment, space1, u32)),
        |(_, _, memory_segment, _, address)| {
            Some(Operation::Pop(Address {
                memory_segment,
                address,
            }))
        },
    )(i)
}

fn parse_unary_operations(i: &str) -> IResult<&str, Option<Operation>> {
    map(alt((tag("neg"), tag("not"))), |tag| {
        Some(match tag {
            "neg" => Operation::Neg,
            "not" => Operation::Not,
            _ => panic!("Unknown unary instruction {}", tag),
        })
    })(i)
}

fn parse_binary_operations(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        alt((
            tag("add"),
            tag("sub"),
            tag("eq"),
            tag("gt"),
            tag("lt"),
            tag("and"),
            tag("or"),
        )),
        |operation| {
            let op = match operation {
                "add" => Operation::Add,
                "sub" => Operation::Sub,
                "eq" => Operation::Eq,
                "gt" => Operation::Gt,
                "lt" => Operation::Lt,
                "and" => Operation::And,
                "or" => Operation::Or,
                _ => panic!("Unknown binary operation {}", operation),
            };
            Some(op)
        },
    )(i)
}

fn parse_memory_segment(i: &str) -> IResult<&str, MemorySegment> {
    map(
        alt((
            tag("arguments"),
            tag("local"),
            tag("static"),
            tag("constant"),
            tag("this"),
            tag("that"),
            tag("pointer"),
            tag("temp"),
        )),
        |tag| match tag {
            "arguments" => MemorySegment::Arguments,
            "local" => MemorySegment::Local,
            "static" => MemorySegment::Static,
            "constant" => MemorySegment::Constant,
            "this" => MemorySegment::This,
            "that" => MemorySegment::That,
            "pointer" => MemorySegment::Pointer,
            "temp" => MemorySegment::Temp,
            _ => panic!("Unknown memory segment tag {}", tag),
        },
    )(i)
}

pub fn parse_comment(i: &str) -> IResult<&str, Option<Operation>> {
    map(tuple((space0, tag("//"), not_line_ending)), |_| None)(i)
}

pub fn parse_empty_lines(i: &str) -> IResult<&str, Option<Operation>> {
    map(all_consuming(alt((multispace0, line_ending))), |_| None)(i)
}

#[test]
fn test_parser() {
    assert_eq!(
        parser("push constant 4").unwrap(),
        vec![Stmt {
            operation: Operation::Push(Address {
                memory_segment: MemorySegment::Constant,
                address: 4,
            }),
            text: "push constant 4".to_string()
        }]
    );

    assert_eq!(
        parser("pop constant 4").unwrap(),
        vec![Stmt {
            operation: Operation::Pop(Address {
                memory_segment: MemorySegment::Constant,
                address: 4,
            }),
            text: "pop constant 4".to_string()
        }]
    );

    assert_eq!(
        parser("add").unwrap(),
        vec![Stmt {
            operation: Operation::Add,
            text: "add".to_string()
        }]
    );

    let test_script = r#"// Start with a comment

push constant 7
push constant 8
add"#;
    assert_eq!(
        parser(test_script).unwrap(),
        vec![
            Stmt {
                operation: Operation::Push(Address {
                    memory_segment: MemorySegment::Constant,
                    address: 7
                }),
                text: "push constant 7".to_string()
            },
            Stmt {
                operation: Operation::Push(Address {
                    memory_segment: MemorySegment::Constant,
                    address: 8
                }),
                text: "push constant 8".to_string()
            },
            Stmt {
                operation: Operation::Add,
                text: "add".to_string()
            }
        ]
    );

    assert_eq!(
        parser("neg").unwrap(),
        vec![Stmt {
            operation: Operation::Neg,
            text: "neg".to_string()
        }]
    );
    assert_eq!(
        parser("not").unwrap(),
        vec![Stmt {
            operation: Operation::Not,
            text: "not".to_string()
        }]
    );
}
