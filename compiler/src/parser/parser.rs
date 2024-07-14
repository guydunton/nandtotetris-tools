use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, map_opt, opt, value};
use nom::error::{context, VerboseError};
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
    Class, ClassVariable, ClassVariableVisibility, CompiledClass, IfDetails, LetDetails,
    ReturnType, Statement, Subroutine, SubroutineType, Variable, VariableRef, VariableType,
    WhileDetails, AST,
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
    let (s, var_type) = cut(context(
        "variable type",
        terminated(var_type, all_whitespace1),
    ))(s)?;

    let (s, first_var_name) = cut(parse_identifier)(s)?;

    let (s, other_vars) = cut(fold_many0(
        tuple((char(','), all_whitespace0, parse_identifier)),
        Vec::new,
        |mut acc: Vec<String>, (_, _, var_name)| {
            acc.push(var_name);
            acc
        },
    ))(s)?;

    let (s, _) = cut(preceded(all_whitespace0, char(';')))(s)?;

    let mut var_details =
        Statement::var().add_var(Variable::new(&first_var_name, var_type.clone()));

    for var in other_vars {
        var_details = var_details.add_var(Variable::new(&var, var_type.clone()));
    }

    Ok((s, var_details.as_statement()))
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
    let (s, condition) = context("if condition", cut(parse_expression))(s)?;
    let (s, _) = cut(tuple((
        all_whitespace0,
        char(')'),
        all_whitespace0,
        char('{'),
    )))(s)?;
    let (s, if_body) = cut(parse_statements)(s)?;
    let (s, _) = cut(char('}'))(s)?;
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
    let (s, identifier) = cut(alt((
        parse_indexed_identifier,
        map(parse_identifier, |name| VariableRef::new(&name)),
    )))(s)?;

    let (s, _) = cut(delimited(all_whitespace0, char('='), all_whitespace0))(s)?;
    let (s, expression) = cut(parse_expression)(s)?;
    let (s, _) = cut(preceded(all_whitespace0, char(';')))(s)?;

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

    Ok((s, Variable::new(&identifier, var_type)))
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
        Subroutine::new(&function_name)
            .return_type(return_type)
            .subroutine_type(subroutine_type)
            .add_parameters(parameters)
            .add_statements(statements),
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
            .map(|identifier| {
                ClassVariable::new(&identifier)
                    .visibility(visibility)
                    .var_type(var_type.clone())
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
        Class::new(&identifier)
            .add_subroutines(subroutines)
            .add_variables(variables.into_iter().flatten().collect()),
    ))
}

pub fn parse_jack(files: Vec<FileInput>) -> Result<AST, String> {
    let mut result = Vec::with_capacity(files.len());
    for file in files {
        let input = Span::new(&file.contents);
        let output = all_consuming(parse_class)(input);

        match output.finish() {
            Ok(compiled_class) => result.push(CompiledClass {
                class: compiled_class.1,
                source_filename: file.filename,
            }),
            Err(e) => {
                return Err(format!(
                    "Failed to compile with error in file {}:\n{}",
                    file.filename,
                    e.to_string()
                ));
            }
        }
    }
    Ok(AST { classes: result })
}
