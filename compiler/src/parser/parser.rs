use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, map_opt, opt, value};
use nom::error::{context, convert_error, VerboseError};
use nom::multi::{fold_many0, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{Finish, IResult};

use super::expression::parse_expression;
use super::parse_utils::{
    all_whitespace0, all_whitespace1, parse_identifier, parse_indexed_identifier,
    parse_subroutine_call,
};
use super::Span;

use crate::ast::{
    Class, ClassVariable, ClassVariableVisibility, IfDetails, LetDetails, ReturnType, Statement,
    Subroutine, SubroutineType, Variable, VariableRef, VariableType, WhileDetails,
};

pub struct FileInput {
    filename: String,
    contents: String,
}

impl FileInput {
    pub fn new(filename: &str, contents: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            contents: contents.to_owned(),
        }
    }
}

fn parse_return_type(i: Span) -> IResult<Span, ReturnType, VerboseError<Span>> {
    map(parse_identifier, |name| match name.as_str() {
        "void" => ReturnType::Void,
        "bool" => ReturnType::Boolean,
        "char" => ReturnType::Char,
        "int" => ReturnType::Int,
        _ => ReturnType::ClassName(name),
    })(i)
}

fn var_type(i: Span) -> IResult<Span, VariableType, VerboseError<Span>> {
    map_opt(parse_identifier, |name| match name.as_str() {
        "Array" => Some(VariableType::Array),
        "boolean" => Some(VariableType::Boolean),
        "char" => Some(VariableType::Char),
        "int" => Some(VariableType::Int),
        _ => Some(VariableType::ClassName(name)),
    })(i)
}

fn parse_var_decl(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = terminated(tag("var"), all_whitespace1)(i)?;
    let (s, var_type) = terminated(var_type, all_whitespace1)(s)?;

    let (s, first_var_name) = parse_identifier(s)?;

    let (s, other_vars) = fold_many0(
        tuple((char(','), all_whitespace0, parse_identifier)),
        Vec::new,
        |mut acc: Vec<String>, (_, _, var_name)| {
            acc.push(var_name);
            acc
        },
    )(s)?;

    let (s, _) = preceded(all_whitespace0, char(';'))(s)?;

    let mut variables = Vec::with_capacity(other_vars.len() + 1);
    variables.push(Variable {
        identifier: first_var_name,
        var_type: var_type.clone(),
    });

    other_vars.into_iter().for_each(|var| {
        variables.push(Variable {
            identifier: var,
            var_type: var_type.clone(),
        });
    });

    Ok((s, Statement::VarDecl(variables)))
}

fn parse_return(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = tag("return")(i)?;
    let (s, expr) = opt(delimited(
        all_whitespace0,
        parse_expression,
        all_whitespace0,
    ))(s)?;
    let (s, _) = char(';')(s)?;
    Ok((s, Statement::Return(expr)))
}

fn parse_else(i: Span) -> IResult<Span, Vec<Statement>, VerboseError<Span>> {
    let (s, _) = tuple((all_whitespace0, tag("else"), all_whitespace0, char('{')))(i)?;
    let (s, statements) = parse_statements(s)?;
    let (s, _) = char('}')(s)?;

    Ok((s, statements))
}

fn parse_if(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = tuple((tag("if"), all_whitespace0, char('('), all_whitespace0))(i)?;
    let (s, condition) = parse_expression(s)?;
    let (s, _) = tuple((all_whitespace0, char(')'), all_whitespace0, char('{')))(s)?;
    let (s, if_body) = parse_statements(s)?;
    let (s, _) = char('}')(s)?;
    let (s, else_body) = opt(parse_else)(s)?;

    Ok((
        s,
        Statement::If(IfDetails {
            condition,
            if_body,
            else_body,
        }),
    ))
}

fn parse_let(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = terminated(tag("let"), all_whitespace1)(i)?;
    let (s, identifier) = alt((
        parse_indexed_identifier,
        map(parse_identifier, |name| VariableRef { name, index: None }),
    ))(s)?;

    let (s, _) = delimited(all_whitespace0, char('='), all_whitespace0)(s)?;
    let (s, expression) = parse_expression(s)?;
    let (s, _) = preceded(all_whitespace0, char(';'))(s)?;

    Ok((
        s,
        Statement::Let(LetDetails {
            identifier,
            expression,
        }),
    ))
}

fn parse_do(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = tuple((tag("do"), all_whitespace1))(i)?;
    let (s, call) = parse_subroutine_call(s)?;
    let (s, _) = tuple((all_whitespace0, char(';')))(s)?;

    Ok((s, Statement::Do(call)))
}

fn parse_while(i: Span) -> IResult<Span, Statement, VerboseError<Span>> {
    let (s, _) = terminated(tag("while"), all_whitespace0)(i)?;
    let (s, condition) = delimited(
        pair(char('('), all_whitespace0),
        parse_expression,
        pair(all_whitespace0, char(')')),
    )(s)?;

    let (s, _) = pair(all_whitespace0, char('{'))(s)?;
    let (s, body) = parse_statements(s)?;
    let (s, _) = char('}')(s)?;

    Ok((s, Statement::While(WhileDetails { condition, body })))
}

fn parse_statements(i: Span) -> IResult<Span, Vec<Statement>, VerboseError<Span>> {
    // many0(delimited(
    //     all_whitespace0,
    //     alt((
    //         parse_var_decl,
    //         parse_let,
    //         parse_while,
    //         parse_if,
    //         parse_do,
    //         parse_return,
    //     )),
    //     all_whitespace0,
    // ))(i)
    let (s, _) = all_whitespace0(i)?;
    let (s, statements) = context(
        "statement separated list",
        separated_list0(
            context("statement whitespace0", all_whitespace0),
            alt((
                context("var decl", parse_var_decl),
                context("let", parse_let),
                context("while", parse_while),
                context("if", parse_if),
                context("do", parse_do),
                context("return", parse_return),
            )),
        ),
    )(s)?;
    let (s, _) = all_whitespace0(s)?;

    Ok((s, statements))
}

fn parse_parameter(i: Span) -> IResult<Span, Variable, VerboseError<Span>> {
    let (s, var_type) = terminated(var_type, all_whitespace1)(i)?;
    let (s, identifier) = parse_identifier(s)?;

    Ok((
        s,
        Variable {
            identifier,
            var_type,
        },
    ))
}

fn parse_function(i: Span) -> IResult<Span, Subroutine, VerboseError<Span>> {
    let subroutine_type_parser = alt((
        value(SubroutineType::Function, tag("function")),
        value(SubroutineType::Constructor, tag("constructor")),
        value(SubroutineType::Method, tag("method")),
    ));
    let (s, subroutine_type) = terminated(subroutine_type_parser, all_whitespace1)(i)?;
    let (s, return_type) = terminated(parse_return_type, all_whitespace1)(s)?;
    let (s, function_name) = terminated(parse_identifier, all_whitespace0)(s)?;
    let (s, _) = char('(')(s)?;

    // This needs replacing with parameters
    let (s, parameters) = separated_list0(
        tuple((all_whitespace0, char(','), all_whitespace0)),
        parse_parameter,
    )(s)?;

    let (s, _) = tuple((char(')'), all_whitespace0, char('{')))(s)?;

    let (s, statements) = parse_statements(s)?;

    let (s, _) = char('}')(s)?;

    Ok((
        s,
        Subroutine {
            identifier: function_name,
            return_type,
            parameters,
            statements,
            subroutine_type,
        },
    ))
}

fn parse_class_variable_visibility(
    i: Span,
) -> IResult<Span, ClassVariableVisibility, VerboseError<Span>> {
    alt((
        value(ClassVariableVisibility::Field, tag("field")),
        value(ClassVariableVisibility::Static, tag("static")),
    ))(i)
}

fn parse_variable(i: Span) -> IResult<Span, Vec<ClassVariable>, VerboseError<Span>> {
    let (s, visibility) = terminated(parse_class_variable_visibility, all_whitespace1)(i)?;
    let (s, var_type) = terminated(var_type, all_whitespace1)(s)?;
    let (s, identifiers) = separated_list1(
        tuple((all_whitespace0, char(','), all_whitespace0)),
        parse_identifier,
    )(s)?;
    let (s, _) = pair(all_whitespace0, char(';'))(s)?;

    Ok((
        s,
        identifiers
            .into_iter()
            .map(|identifier| ClassVariable {
                visibility,
                var_type: var_type.clone(),
                identifier,
            })
            .collect(),
    ))
}

fn parse_class(i: Span) -> IResult<Span, Class, VerboseError<Span>> {
    let (s, _) = all_whitespace0(i)?;
    let (s, _) = terminated(tag("class"), all_whitespace0)(s)?;
    let (s, identifier) = terminated(parse_identifier, all_whitespace0)(s)?;

    let (s, _) = terminated(tag("{"), all_whitespace0)(s)?;

    let (s, variables) =
        separated_list0(all_whitespace0, context("class variables", parse_variable))(s)?;
    let (s, _) = all_whitespace0(s)?;
    let (s, subroutines) = separated_list0(all_whitespace1, parse_function)(s)?;

    let (s, _) = delimited(all_whitespace0, tag("}"), all_whitespace0)(s)?;

    Ok((
        s,
        Class {
            identifier,
            subroutines,
            variables: variables.into_iter().flatten().collect(),
        },
    ))
}

pub fn parse_jack(files: Vec<FileInput>) -> Result<Vec<(Class, String)>, String> {
    let mut result = Vec::with_capacity(files.len());
    for file in files {
        let input = Span::new(&file.contents);
        let output = all_consuming(parse_class)(input);

        match output {
            Ok(compiled_class) => result.push((compiled_class.1, file.filename)),
            Err(span) => {
                return Err(format!(
                    "Failed to compile with error in file {}:\n{}",
                    file.filename,
                    span.to_string()
                ))
            }
        }
    }
    Ok(result)
}