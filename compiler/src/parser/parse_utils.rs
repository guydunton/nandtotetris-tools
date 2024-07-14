use crate::ast::{Expr, SubroutineCall, VariableRef};

use super::expression::parse_expression;
use super::Span;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace1};
use nom::combinator::value;
use nom::error::VerboseError;
use nom::multi::{fold_many0, fold_many1, many0, separated_list0};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;
use nom::Parser;

pub fn parse_indexed_identifier(i: Span) -> IResult<Span, VariableRef, VerboseError<Span>> {
    let (s, name) = parse_identifier(i)?;
    let (s, _) = delimited(all_whitespace0, char('['), all_whitespace0)(s)?;
    let (s, sub_expr) = parse_expression(s)?;
    let (s, _) = delimited(all_whitespace0, char(']'), all_whitespace0)(s)?;

    Ok((s, VariableRef::new_with_index(&name, sub_expr)))
}

pub fn parse_identifier(i: Span) -> IResult<Span, String, VerboseError<Span>> {
    let (s, part1) = alt((alpha1, tag("_")))(i)?;
    let (s, part2) = many0(alt((alphanumeric1, tag("_"))))(s)?;

    let mut part1_str = part1.to_string();
    let part2_str = part2
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("");

    part1_str.push_str(&part2_str);

    Ok((s, part1_str))
}

fn comment(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    value((), tuple((tag("//"), is_not("\n"), multispace1))).parse(i)
}

fn multiline_comment(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/")))).parse(i)
}

fn whitespace(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    value((), multispace1).parse(i)
}

pub fn all_whitespace1(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    fold_many1(
        alt((multiline_comment, comment, whitespace)),
        || (),
        |_, _| (),
    )(i)
}

pub fn all_whitespace0(i: Span) -> IResult<Span, (), VerboseError<Span>> {
    fold_many0(
        alt((multiline_comment, comment, whitespace)),
        || (),
        |_, _| (),
    )(i)
}

fn parse_parameter_list(i: Span) -> IResult<Span, Vec<Expr>, VerboseError<Span>> {
    separated_list0(
        char(','),
        delimited(all_whitespace0, parse_expression, all_whitespace0),
    )(i)
}

fn parse_function_call(i: Span) -> IResult<Span, SubroutineCall, VerboseError<Span>> {
    let (s, subroutine_name) = parse_identifier(i)?;
    let (s, _) = char('(')(s)?;
    let (s, parameters) = parse_parameter_list(s)?;
    let (s, _) = char(')')(s)?;

    Ok((
        s,
        SubroutineCall::new()
            .name(&subroutine_name)
            .add_parameters(parameters),
    ))
}

fn parse_method_call(i: Span) -> IResult<Span, SubroutineCall, VerboseError<Span>> {
    let (s, type_name) = terminated(parse_identifier, char('.'))(i)?;
    let (s, subroutine_name) = parse_identifier(s)?;
    let (s, _) = char('(')(s)?;
    let (s, parameters) = parse_parameter_list(s)?;
    let (s, _) = char(')')(s)?;

    Ok((
        s,
        SubroutineCall::new()
            .name(&subroutine_name)
            .set_target(&type_name)
            .add_parameters(parameters),
    ))
}

pub fn parse_subroutine_call(i: Span) -> IResult<Span, SubroutineCall, VerboseError<Span>> {
    alt((parse_function_call, parse_method_call))(i)
}
