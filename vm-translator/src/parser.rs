use crate::ast::{Address, Function, MemorySegment, Operation, Stmt};
use nom::character::complete::{
    anychar, line_ending, multispace0, not_line_ending, space0, space1, u32,
};
use nom::combinator::{all_consuming, eof};
use nom::multi::many_till;
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
        parse_label,
        parse_goto,
        parse_if_goto,
        parse_function,
        parse_binary_operations,
        parse_unary_operations,
        parse_comment,
        parse_empty_lines,
    ))(i)
}

fn parse_push(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((
            space0,
            tag("push"),
            space1,
            parse_memory_segment,
            space1,
            u32,
        )),
        |(_, _, _, memory_segment, _, address)| {
            Some(Operation::Push(Address {
                memory_segment,
                address,
            }))
        },
    )(i)
}

fn parse_pop(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((
            space0,
            tag("pop"),
            space1,
            parse_memory_segment,
            space1,
            u32,
        )),
        |(_, _, _, memory_segment, _, address)| {
            Some(Operation::Pop(Address {
                memory_segment,
                address,
            }))
        },
    )(i)
}

fn parse_label(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((space0, tag("label"), space1, parse_name)),
        |(_, _, _, name)| Some(Operation::Label(name)),
    )(i)
}

fn parse_if_goto(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((space0, tag("if-goto"), space1, parse_name)),
        |(_, _, _, name)| Some(Operation::ConditionalJump(name)),
    )(i)
}

fn parse_goto(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((space0, tag("goto"), space1, parse_name)),
        |(_, _, _, name)| Some(Operation::Jump(name)),
    )(i)
}

fn parse_function(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((tag("function"), space1, parse_name, u32)),
        |(_, _, name, num_locals)| Some(Operation::Function(Function { name, num_locals })),
    )(i)
}

fn parse_unary_operations(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((space0, alt((tag("neg"), tag("not"))))),
        |(_, tag)| {
            Some(match tag {
                "neg" => Operation::Neg,
                "not" => Operation::Not,
                _ => panic!("Unknown unary instruction {}", tag),
            })
        },
    )(i)
}

fn parse_binary_operations(i: &str) -> IResult<&str, Option<Operation>> {
    map(
        tuple((
            space0,
            alt((
                tag("add"),
                tag("sub"),
                tag("eq"),
                tag("gt"),
                tag("lt"),
                tag("and"),
                tag("or"),
            )),
        )),
        |(_, operation)| {
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
            tag("argument"),
            tag("local"),
            tag("static"),
            tag("constant"),
            tag("this"),
            tag("that"),
            tag("pointer"),
            tag("temp"),
        )),
        |tag| match tag {
            "argument" => MemorySegment::Arguments,
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

fn parse_comment(i: &str) -> IResult<&str, Option<Operation>> {
    map(tuple((space0, tag("//"), not_line_ending)), |_| None)(i)
}

fn parse_empty_lines(i: &str) -> IResult<&str, Option<Operation>> {
    map(all_consuming(alt((multispace0, line_ending))), |_| None)(i)
}

fn parse_name(i: &str) -> IResult<&str, String> {
    map(many_till(anychar, alt((space1, eof))), |(text, _)| {
        text.into_iter().collect()
    })(i)
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

#[test]
fn test_parser_labels() {
    assert_eq!(
        parser("label LOOP").unwrap(),
        vec![Stmt {
            operation: Operation::Label("LOOP".to_owned()),
            text: "label LOOP".to_owned()
        }]
    );

    assert_eq!(
        parser("\tlabel Math.test").unwrap()[0].operation,
        Operation::Label("Math.test".to_owned())
    );
}

#[test]
fn test_parser_with_spaces() {
    assert!(parser("\teq").is_ok());
    assert!(parser("\tpop local 0").is_ok());
    assert!(parser("\tpush constant 0").is_ok());
    assert!(parser("\tnot").is_ok());
}

#[test]
fn test_comment_parsing() {
    assert!(parser("\t// This is my comment").is_ok());
    assert_eq!(
        parser("push constant 2 // This is a comment").unwrap(),
        vec![Stmt {
            operation: Operation::Push(Address {
                memory_segment: MemorySegment::Constant,
                address: 2,
            }),
            text: "push constant 2 // This is a comment".to_owned()
        }]
    );
}

#[test]
fn test_if_goto_parsing() {
    assert_eq!(
        parser("if-goto LOOP").unwrap()[0].operation,
        Operation::ConditionalJump("LOOP".to_owned())
    );
}

#[test]
fn test_goto_parsing() {
    assert_eq!(
        parser("goto LOOP").unwrap()[0].operation,
        Operation::Jump("LOOP".to_owned())
    );
}

#[test]
fn test_function_parsing() {
    assert_eq!(
        parser("function myfunc 3").unwrap()[0].operation,
        Operation::Function(Function {
            name: "myfunc".to_owned(),
            num_locals: 3,
        })
    );
}
