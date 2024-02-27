//! In this example we build an [S-expression](https://en.wikipedia.org/wiki/S-expression)
//! parser and tiny [lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language)) interpreter.
//! Lisp is a simple type of language made up of Atoms and Lists, forming easily parsable trees.
use super::{Atom, Expr};
use crate::strings;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char, hex_digit1, multispace0, satisfy},
    combinator::{all_consuming, cut, map, map_opt, map_parser, map_res, recognize, rest},
    error::{context, VerboseError},
    multi::{many0, many1, many1_count},
    number::complete::double,
    sequence::{delimited, preceded, terminated},
    AsChar, IResult, Parser,
};
use uuid::Uuid;

/// Reserved words true and false.
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
    let mut parser = map(rest, |sym_str: &str| Atom::Symbol(sym_str.to_string()));
    parser.parse(i)
}

/// A float.
fn parse_num(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(double, Atom::Num).parse(i)
}

/// A hex number starting with 0x.
fn parse_bits(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let digits = many1(satisfy(|x| x.is_hex_digit() || x == '_'));
    let hex = map_opt(digits, |s| {
        u64::from_str_radix(&s.into_iter().filter(|x| *x != '_').collect::<String>(), 16).ok()
    });
    map(preceded(tag("0x"), hex), Atom::Bits).parse(i)
}

/// Standard UUID syntax.
fn parse_uuid(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let segment = char('-').and(hex_digit1);
    map_res(
        recognize(hex_digit1.and(many1_count(segment))),
        |num_str: &str| Uuid::try_parse(num_str).map(Atom::Uuid),
    )
    .parse(i)
}

/// A scalar is a symbol or one of several types of number
/// These are matched in most specific to least specific order.
/// All the text up to a bracket or whitespace will be matched.
fn parse_scalar(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    let mut parser = map_parser(
        is_not(" \t\r\n)("),
        all_consuming(
            parse_uuid
                .or(parse_bits)
                .or(parse_num)
                .or(parse_bool)
                .or(parse_symbol),
        ),
    );
    parser.parse(i)
}

/// Quoted string.
fn parse_string(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(strings::parse_string, Atom::Str).parse(i)
}

/// An atom is a quoted string or a scalar.
fn parse_atom(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    parse_string.or(parse_scalar).parse(i)
}

/// An atom is a constant expression
fn parse_constant(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(parse_atom, Expr::Constant).parse(i)
}

/// A list is zero or more expressions in brackets.
fn parse_list(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        delimited(
            char('('),
            parse_bare_list,
            context("closing paren", cut(char(')'))),
        ),
        Expr::list,
    )
    .parse(i)
}

/// An expression is either a list or a constant
fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    parse_list.or(parse_constant).parse(i)
}

/// An unbracketed list of zero or more expressions.
fn parse_bare_list(i: &str) -> IResult<&str, Vec<Expr>, VerboseError<&str>> {
    preceded(multispace0, many0(terminated(parse_expr, multispace0))).parse(i)
}

/// The parser accepts a single expression,
/// usually a a bracketed list.
pub fn parse_s_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    all_consuming(delimited(multispace0, parse_expr, multispace0)).parse(i)
}

/// Not Working!
///
/// The parser accepts a single atom returning Expr::Constant
/// or an unbracketed or bracketed list returning Expr::List.
/// Empty input (or whitespace) returns and empty Expr::List.
pub fn parse_s_exprs(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(all_consuming(parse_bare_list), |mut xs| {
        if xs.len() == 1 {
            xs.remove(0)
        } else {
            Expr::list(xs)
        }
    })
    .parse(i)
}
