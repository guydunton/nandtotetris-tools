use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{alpha1, alphanumeric1, multispace0, multispace1};
use nom::combinator::{map, opt, recognize, value};
use nom::multi::{fold_many0, many0, many0_count};
use nom::sequence::{delimited, pair};
use nom::sequence::{terminated, tuple};
use nom::{IResult, Parser};
use nom_locate::LocatedSpan;

use crate::ast::Class;

pub type Span<'a> = LocatedSpan<&'a str>;

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

fn comment(i: Span) -> IResult<Span, ()> {
    value((), tuple((tag("//"), is_not("\n"), multispace1))).parse(i)
}

fn multiline_comment(i: Span) -> IResult<Span, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/")))).parse(i)
}

fn whitespace(i: Span) -> IResult<Span, ()> {
    value((), multispace1).parse(i)
}

fn all_whitespace(i: Span) -> IResult<Span, ()> {
    fold_many0(
        alt((multiline_comment, comment, whitespace)),
        || (),
        |_, _| (),
    )(i)
}

fn parse_identifier(i: Span) -> IResult<Span, String> {
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

fn parse_class(i: Span) -> IResult<Span, Class> {
    let (s, _) = all_whitespace(i)?;
    let (s, _) = terminated(tag("class"), all_whitespace)(s)?;
    let (s, identifier) = terminated(parse_identifier, opt(all_whitespace))(s)?;

    // all_whitespace needs replacing with the correct parser
    let (s, _) = delimited(tag("{"), all_whitespace, tag("}"))(s)?;

    Ok((
        s,
        Class {
            identifier,
            subroutines: vec![],
        },
    ))
}

pub fn parse_jack(files: Vec<FileInput>) -> Result<Vec<Class>, String> {
    for file in files {
        let input = Span::new(&file.contents);
        let output = parse_class(input);

        println!("output: {:?}", output);
    }
    Ok(Vec::new())
}
