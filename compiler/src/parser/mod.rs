mod expression;
mod parse_utils;
mod parser;

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub use parser::{parse_jack, FileInput};
