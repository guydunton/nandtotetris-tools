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

fn parse_binary_operation(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, lhs) = context("binary-op lhs", parse_sub_expression)(i)?;
    let (s, operator) = delimited(all_whitespace0, parse_binary_operator, all_whitespace0)(s)?;
    let (s, rhs) = context("binary-op rhs", parse_expression)(s)?;

    Ok((
        s,
        Expr::BinaryExpr {
            lhs: Box::new(lhs),
            op: operator,
            rhs: Box::new(rhs),
        },
    ))
}

fn parse_brackets(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    let (s, _) = char('(')(i)?;
    let (s, expr) = cut(context("parsing bracketed expression", parse_expression))(s)?;
    let (s, _) = cut(char(')'))(s)?;

    Ok((s, Expr::BracketedExpr(Box::new(expr))))
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
    let (s, identifier) = parse_identifier(i)?;
    let (s, _) = delimited(all_whitespace0, char('['), all_whitespace0)(s)?;
    let (s, index) = cut(context("index expression", parse_expression))(s)?;
    let (s, _) = cut(delimited(all_whitespace0, char(']'), all_whitespace0))(s)?;

    Ok((
        s,
        Expr::VarRef(VariableRef::new_with_index(&identifier, index)),
    ))
}

fn parse_sub_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    context(
        "sub-expression",
        alt((
            parse_brackets,
            parse_unary_op,
            map(parse_subroutine_call, |details| Expr::Call(details)),
            parse_constant,
            parse_indexed_identifier,
            map(parse_identifier, |name| {
                Expr::VarRef(VariableRef::new(&name))
            }),
        )),
    )(i)
}

pub fn parse_expression(i: Span) -> IResult<Span, Expr, VerboseError<Span>> {
    context(
        "expression",
        alt((
            parse_binary_operation,
            parse_brackets,
            parse_unary_op,
            map(parse_subroutine_call, |details| Expr::Call(details)),
            parse_constant,
            parse_indexed_identifier,
            map(parse_identifier, |name| {
                Expr::VarRef(VariableRef::new(&name))
            }),
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
        Expr::VarRef(VariableRef::new("i"))
    );
    assert_eq!(
        expr(parse_expression(span("i < 3"))),
        Expr::BinaryExpr {
            lhs: Box::new(Expr::VarRef(VariableRef::new("i"))),
            op: crate::ast::BinaryOp::Lt,
            rhs: Box::new(Expr::Constant(Constant::Int(3)))
        }
    );
    assert_eq!(
        expr(parse_expression(span("a[ i + 1 ]"))),
        Expr::VarRef(VariableRef::new_with_index(
            "a",
            Expr::BinaryExpr {
                lhs: Box::new(Expr::VarRef(VariableRef::new("i"))),
                op: BinaryOp::Plus,
                rhs: Box::new(Expr::Constant(Constant::Int(1)))
            }
        ))
    );
    assert_eq!(
        expr(parse_expression(span("read()"))),
        Expr::Call(crate::ast::SubroutineCall::new().name("read"))
    );

    assert_eq!(
        expr(parse_expression(span("-i"))),
        Expr::UnaryExpr(
            UnaryOp::Minus,
            Box::new(Expr::VarRef(VariableRef::new("i")))
        )
    );

    assert_eq!(
        expr(parse_expression(span("~(b | c)"))),
        Expr::UnaryExpr(
            UnaryOp::Not,
            Box::new(Expr::BracketedExpr(Box::new(Expr::BinaryExpr {
                lhs: Box::new(Expr::VarRef(VariableRef::new("b"))),
                op: BinaryOp::Or,
                rhs: Box::new(Expr::VarRef(VariableRef::new("c")))
            })))
        )
    );

    assert_eq!(
        expr(parse_expression(span(
            "(((y + size) < 254) & ((x + size) < 510))"
        ))),
        Expr::BracketedExpr(Box::new(Expr::BinaryExpr {
            // ((y + size) < 254)
            lhs: Box::new(Expr::BracketedExpr(Box::new(
                // (y + size) < 254
                Expr::BinaryExpr {
                    // (y + size)
                    lhs: Box::new(Expr::BracketedExpr(Box::new(Expr::binary_op(
                        Expr::VarRef(VariableRef::new("y")),
                        BinaryOp::Plus,
                        Expr::VarRef(VariableRef::new("size"))
                    )))),
                    op: BinaryOp::Lt,
                    rhs: Box::new(Expr::Constant(Constant::Int(254)))
                }
            ))),
            op: BinaryOp::And,
            // ((x + size) < 510)
            rhs: Box::new(Expr::BracketedExpr(Box::new(Expr::BinaryExpr {
                lhs: Box::new(Expr::BracketedExpr(Box::new(Expr::BinaryExpr {
                    lhs: Box::new(Expr::VarRef(VariableRef::new("x"))),
                    op: BinaryOp::Plus,
                    rhs: Box::new(Expr::VarRef(VariableRef::new("size")))
                }))),
                op: BinaryOp::Lt,
                rhs: Box::new(Expr::Constant(Constant::Int(510)))
            })))
        }))
    );

    assert_eq!(expr(parse_expression(span("true"))), Expr::true_c());
}
