use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{cut, map, value};
use nom::error::{context, VerboseError};
use nom::sequence::delimited;
use nom::IResult;
use nom_locate::LocatedSpan;

use crate::ast::{BinaryOp, Constant, Expr, KeywordConstant, UnaryOp, VariableRef};

use super::parse_utils::{all_whitespace0, parse_identifier, parse_subroutine_call};
use super::Span;

use nom::bytes::complete::{tag, take_while};

fn parse_constant(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    fn is_not_quote(c: char) -> bool {
        return c != '"';
    }

    alt((
        context(
            "string constant",
            map(
                delimited(char('\"'), take_while(is_not_quote), char('\"')),
                |s: LocatedSpan<&str>| Expr::Constant(Constant::String(s.to_string())),
            ),
        ),
        context(
            "integer constant",
            map(nom::character::complete::i32, |val| {
                Expr::Constant(Constant::Int(val))
            }),
        ),
        context(
            "keyword constant",
            map(
                alt((
                    value(KeywordConstant::False, tag("false")),
                    value(KeywordConstant::True, tag("true")),
                    value(KeywordConstant::Null, tag("null")),
                    value(KeywordConstant::This, tag("this")),
                )),
                |keyword| Expr::Constant(Constant::Keyword(keyword)),
            ),
        ),
    ))(i)
}

fn parse_binary_operator(i: Span) -> IResult<Span, BinaryOp, VerboseError<Span>> {
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
    let (s, operator) = delimited(all_whitespace0, parse_binary_operator, all_whitespace0)(s)?;
    let (s, rhs) = parse_expression(s)?;

    Ok((s, Expr::MultiExpr(Box::new(lhs), vec![(operator, rhs)])))
}

fn parse_unary_op(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, operator) = alt((
        value(UnaryOp::Minus, char('-')),
        value(UnaryOp::Not, char('~')),
    ))(i)?;

    let (s, expr) = cut(context("Unary expression", parse_expression))(s)?;

    Ok((s, Expr::UnaryExpr(operator, Box::new(expr))))
}

fn parse_indexed_identifier(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, name) = parse_identifier(i)?;
    let (s, _) = delimited(all_whitespace0, char('['), all_whitespace0)(s)?;
    let (s, sub_expr) = cut(context("index expression", parse_expression))(s)?;
    let (s, _) = cut(delimited(all_whitespace0, char(']'), all_whitespace0))(s)?;

    Ok((
        s,
        Expr::VarRef(VariableRef {
            name,
            index: Some(Box::new(sub_expr)),
        }),
    ))
}

fn parse_parentheses(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, _) = char('(')(i)?;
    let (s, expr) = cut(parse_expression)(s)?;
    let (s, _) = cut(char(')'))(s)?;
    Ok((s, expr))
}

fn parse_sub_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    context(
        "expression",
        alt((
            parse_unary_op,
            map(parse_subroutine_call, |details| Expr::Call(details)),
            parse_parentheses,
            parse_indexed_identifier,
            map(parse_identifier, |name| {
                Expr::VarRef(VariableRef { name, index: None })
            }),
            parse_constant,
        )),
    )(i)
}

pub fn parse_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    context(
        "expression",
        alt((
            parse_unary_op,
            parse_operation,
            map(parse_subroutine_call, |details| Expr::Call(details)),
            parse_parentheses,
            parse_indexed_identifier,
            map(parse_identifier, |name| {
                Expr::VarRef(VariableRef { name, index: None })
            }),
            parse_constant,
        )),
    )(i)
}

#[test]
fn test_expression() {
    let expr = |r: IResult<Span, Expr, VerboseError<Span>>| r.unwrap().1;
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

    assert_eq!(
        expr(parse_expression(span("-i"))),
        Expr::UnaryExpr(
            UnaryOp::Minus,
            Box::new(Expr::VarRef(VariableRef {
                name: "i".to_owned(),
                index: None
            }))
        )
    );

    assert_eq!(
        expr(parse_expression(span("~(b | c)"))),
        Expr::UnaryExpr(
            UnaryOp::Not,
            Box::new(Expr::MultiExpr(
                Box::new(Expr::VarRef(VariableRef {
                    name: "b".to_owned(),
                    index: None
                })),
                vec![(
                    BinaryOp::Or,
                    Expr::VarRef(VariableRef {
                        name: "c".to_owned(),
                        index: None
                    })
                )]
            ))
        )
    );
}
