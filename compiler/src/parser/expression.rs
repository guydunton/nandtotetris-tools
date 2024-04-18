use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{map, value};
use nom::error::VerboseError;
use nom::sequence::delimited;
use nom::IResult;
use nom_locate::LocatedSpan;

use crate::ast::{BinaryOp, Constant, Expr, KeywordConstant, VariableRef};

use super::parse_utils::{all_whitespace0, parse_identifier, parse_subroutine_call};
use super::Span;

use nom::bytes::complete::{tag, take_while};

fn parse_constant(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    fn is_not_quote(c: char) -> bool {
        return c != '"';
    }

    alt((
        map(
            delimited(char('\"'), take_while(is_not_quote), char('\"')),
            |s: LocatedSpan<&str>| Expr::Constant(Constant::String(s.to_string())),
        ),
        map(nom::character::complete::i32, |val| {
            Expr::Constant(Constant::Int(val))
        }),
        map(
            alt((
                value(KeywordConstant::False, tag("false")),
                value(KeywordConstant::True, tag("true")),
                value(KeywordConstant::Null, tag("null")),
                value(KeywordConstant::This, tag("this")),
            )),
            |keyword| Expr::Constant(Constant::Keyword(keyword)),
        ),
    ))(i)
}

fn parse_operator(i: Span) -> IResult<Span, BinaryOp, VerboseError<Span>> {
    alt((
        value(BinaryOp::Lt, char('<')),
        value(BinaryOp::Gt, char('>')),
        value(BinaryOp::Plus, char('+')),
        value(BinaryOp::Minus, char('-')),
        value(BinaryOp::Mult, char('*')),
        value(BinaryOp::Div, char('/')),
        value(BinaryOp::And, char('&')),
        value(BinaryOp::Or, char('|')),
        value(BinaryOp::Eq, char('=')),
    ))(i)
}

fn parse_operation(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, lhs) = parse_sub_expression(i)?;
    let (s, operator) = delimited(all_whitespace0, parse_operator, all_whitespace0)(s)?;
    let (s, rhs) = parse_sub_expression(s)?;

    Ok((s, Expr::MultiExpr(Box::new(lhs), vec![(operator, rhs)])))
}

fn parse_sub_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    alt((
        map(parse_subroutine_call, |details| Expr::Call(details)),
        parse_constant,
        parse_indexed_identifier,
        map(parse_identifier, |name| {
            Expr::VarRef(VariableRef { name, index: None })
        }),
    ))(i)
}

fn parse_indexed_identifier(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, name) = parse_identifier(i)?;
    let (s, _) = delimited(all_whitespace0, char('['), all_whitespace0)(s)?;
    let (s, sub_expr) = parse_expression(s)?;
    let (s, _) = delimited(all_whitespace0, char(']'), all_whitespace0)(s)?;

    Ok((
        s,
        Expr::VarRef(VariableRef {
            name,
            index: Some(Box::new(sub_expr)),
        }),
    ))
}

pub fn parse_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    alt((parse_operation, parse_sub_expression))(i)
}
/*
#[test]
fn test_expression() {
    let expr = |r: IResult<Span, Expr>| r.unwrap().1;
    let span = |val| Span::new(val);

    assert_eq!(
        expr(parse_expression(span("3"))),
        Expr::Constant(Constant::Int(3))
    );
    assert_eq!(
        expr(parse_expression(span("i"))),
        Expr::VarRef(VariableRef {
            name: "i".to_owned(),
            index: None
        })
    );
    assert_eq!(
        expr(parse_expression(span("i < 3"))),
        Expr::MultiExpr(
            Box::new(Expr::VarRef(VariableRef {
                name: "i".to_owned(),
                index: None
            })),
            vec![(crate::ast::BinaryOp::Lt, Expr::Constant(Constant::Int(3)))]
        )
    );
    assert_eq!(
        expr(parse_expression(span("a[ i + 1 ]"))),
        Expr::VarRef(VariableRef {
            name: "a".to_owned(),
            index: Some(Box::new(Expr::MultiExpr(
                Box::new(Expr::VarRef(VariableRef {
                    name: "i".to_owned(),
                    index: None
                })),
                vec![(BinaryOp::Plus, Expr::Constant(Constant::Int(1)))]
            ))),
        })
    );
    assert_eq!(
        expr(parse_expression(span("read()"))),
        Expr::Call(crate::ast::SubroutineCall {
            type_name: None,
            parameters: vec![],
            subroutine_name: "read".to_owned()
        })
    );
}
*/
