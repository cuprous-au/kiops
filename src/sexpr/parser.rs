//! In this example we build an [S-expression](https://en.wikipedia.org/wiki/S-expression)
//! parser and tiny [lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language)) interpreter.
//! Lisp is a simple type of language made up of Atoms and Lists, forming easily parsable trees.
use super::{Atom, Expr};
use crate::strings;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char, hex_digit1, multispace0, satisfy},
    combinator::{cut, map, map_opt, map_res, recognize},
    error::{context, VerboseError},
    multi::{many0, many1, many1_count},
    number::complete::double,
    sequence::{delimited, preceded},
    AsChar, IResult, Parser,
};
use uuid::Uuid;

/// Continuing the trend of starting from the simplest piece and building up,
/// we start by creating a parser for our atoms.

fn parse_bool(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    alt((
        map(tag("true"), |_| Atom::Bool(true)),
        map(tag("false"), |_| Atom::Bool(false)),
    ))
    .parse(i)
}

/// A symbol.  Any sequence excluding white space and brackets could be a symbol.
/// More specific atoms such as quoted strings or numbers should be matched
/// before symbols.
fn parse_symbol(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let mut parser = map(is_not(" \t\r\n)("), |sym_str: &str| {
        Atom::Symbol(sym_str.to_string())
    });
    parser.parse(i)
}

/// Float parsing.
fn parse_num(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(double, Atom::Num).parse(i)
}

/// Hex parsing.
fn parse_bits(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let digits = many1(satisfy(|x| x.is_hex_digit() || x == '_'));
    let hex = map_opt(digits, |s| {
        u64::from_str_radix(&s.into_iter().filter(|x| *x != '_').collect::<String>(), 16).ok()
    });
    map(preceded(tag("0x"), hex), Atom::Bits).parse(i)
}

fn parse_uuid(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let segment = char('-').and(hex_digit1);
    map_res(
        recognize(hex_digit1.and(many1_count(segment))),
        |num_str: &str| Uuid::try_parse(num_str).map(Atom::Uuid),
    )
    .parse(i)
}

fn parse_string(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(strings::parse_string, Atom::Str).parse(i)
}

/// Now we take all these simple parsers and connect them.
/// We can now parse half of our language!
fn parse_atom(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    parse_string
        .or(parse_uuid)
        .or(parse_bits)
        .or(parse_num)
        .or(parse_bool)
        .or(parse_symbol)
        .parse(i)
}

/// Wrap up an atom as an exporession
fn parse_constant(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(parse_atom, Expr::Constant).parse(i)
}

/// A list of zero or more expressions in brackets.
fn parse_brackets(i: &str) -> IResult<&str, Vec<Expr>, VerboseError<&str>> {
    delimited(
        char('('),
        many0(parse_expr),
        context("closing paren", cut(preceded(multispace0, char(')')))),
    )
    .parse(i)
}

/// Wrap up a list of expressions as an expression.
fn parse_list(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(context("list", parse_brackets), Expr::list).parse(i)
}

/// We tie them all together again, making a top-level expression parser!
/// And that's it!
/// We can now parse our entire lisp language.
pub fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    preceded(multispace0, parse_list.or(parse_constant)).parse(i)
}
