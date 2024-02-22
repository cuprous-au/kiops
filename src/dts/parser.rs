use super::*;
use crate::memo::Memo;
use crate::strings::parse_string;
use nom::{
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, hex_digit1, multispace1, not_line_ending, one_of, satisfy},
    combinator::{cut, eof, map_opt, opt, recognize, value},
    error::VerboseError,
    multi::{many0, many0_count, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult, Parser,
};

/// Parser trait with fixed I and E and a utility method.
pub trait Pex<'a, T>: Parser<&'a str, T, VerboseError<&'a str>> + Sized {
    fn intercept(self) -> Intercept<Self>;
}

impl<'a, T, X> Pex<'a, T> for X
where
    Self: Parser<&'a str, T, VerboseError<&'a str>>,
{
    fn intercept(self) -> Intercept<Self> {
        Intercept(self)
    }
}

/// Intercept the parse method
pub struct Intercept<P>(pub P);

impl<P, I, O, E> Parser<I, O, E> for Intercept<P>
where
    P: Parser<I, O, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        self.0.parse(input)
    }
}

/// Parse a device tree source as a forest of subtrees.
/// At most one will be the root and decendent nodes.
/// The others will be references to labelled nodes.
pub fn tree(input: &str) -> IResult<&str, Vec<Node>, VerboseError<&str>> {
    let item = node().map(Some).or(tag("/dts-v1/;").map(|_| None));
    terminated(spaced_list(item), eof)
        .map(|ns| ns.into_iter().flatten().collect())
        .parse(input)
}

fn nodes<'a>() -> impl Pex<'a, Vec<Node>> {
    Memo::new(|| spaced_list(node()))
}

fn props<'a>() -> impl Pex<'a, Vec<Prop>> {
    spaced_list(prop())
}

fn spaced_list<'a, T>(item: impl Pex<'a, T>) -> impl Pex<'a, Vec<T>> {
    many0(spaced(item))
}

fn spaced<'a, T>(item: impl Pex<'a, T>) -> impl Pex<'a, T> {
    delimited(spacing(), item, spacing())
}

fn block<'a>() -> impl Pex<'a, (Vec<Prop>, Vec<Node>)> {
    delimited(
        tag("{"),
        props().and(Memo::new(nodes)),
        cut(tag("}")).and(spaced(tag(";"))),
    )
}

fn node<'a>() -> impl Pex<'a, Node> {
    let root = value(NodeName::Symbol(Symbol("".into()), None), tag("/"));
    let name = symbol()
        .and(opt(preceded(tag("@"), hex())))
        .map(|(s, a)| NodeName::Symbol(s, a));
    let refer = reference().map(NodeName::Reference);
    let node_name = root.or(name).or(refer);

    opt(label())
        .and(spaced(node_name))
        .and(spaced(block()))
        .map(move |((label, name), (props, nodes))| Node {
            labels: label.into_iter().collect(),
            name,
            props,
            nodes,
        })
}

fn symbol<'a>() -> impl Pex<'a, Symbol> {
    recognize(satisfy(symbol_initial_char).and(take_while(symbol_char)))
        .map(|s: &str| Symbol(s.into()))
}

fn symbol_initial_char(c: char) -> bool {
    match c {
        c if c.is_alphabetic() => true,
        '_' | '$' | '#' => true,
        _ => false,
    }
}

fn symbol_char(c: char) -> bool {
    match c {
        c if c.is_alphanumeric() => true,
        '_' | '$' | '-' | '.' | ',' => true,
        _ => false,
    }
}

fn identifier<'a>() -> impl Pex<'a, Symbol> {
    recognize(satisfy(ident_initial_char).and(take_while(ident_char)))
        .map(|s: &str| Symbol(s.into()))
}

fn ident_initial_char(c: char) -> bool {
    match c {
        c if c.is_alphabetic() => true,
        '_' | '$' => true,
        _ => false,
    }
}

fn ident_char(c: char) -> bool {
    match c {
        c if c.is_alphanumeric() => true,
        '_' | '$' => true,
        _ => false,
    }
}

fn label<'a>() -> impl Pex<'a, Symbol> {
    terminated(identifier(), tag(":"))
}

fn reference<'a>() -> impl Pex<'a, Symbol> {
    preceded(tag("&"), identifier())
}

fn prop<'a>() -> impl Pex<'a, Prop> {
    let non_empty = separated_pair(symbol(), spaced(tag("=")), prop_values())
        .map(|(name, value)| Prop { name, value });
    let empty = symbol().map(|name| Prop {
        name,
        value: [].into(),
    });
    terminated(non_empty.or(empty), spaced(tag(";")))
}

fn prop_values<'a>() -> impl Pex<'a, Vec<Value>> {
    separated_list1(spaced(tag(",")), prop_value())
}

fn prop_value<'a>() -> impl Pex<'a, Value> {
    parse_string
        .map(Value::Text)
        .or(simple_value())
        .or(array().map(Value::Array))
}

fn simple_value<'a>() -> impl Pex<'a, Value> {
    identifier()
        .map(Value::Symbol)
        .or(literal().map(Value::Address))
        .or(reference().map(Value::Reference))
        .or(expr_in_backets().map(Value::Expr))
}

fn literal<'a>() -> impl Pex<'a, Address> {
    preceded(tag("0x"), hex()).or(dec())
}

fn hex<'a>() -> impl Pex<'a, Address> {
    map_opt(hex_digit1, |s: &str| {
        u64::from_str_radix(s, 16).ok().map(Address)
    })
}

fn dec<'a>() -> impl Pex<'a, Address> {
    map_opt(digit1, |s: &str| s.parse::<u64>().ok().map(Address))
}

fn array<'a>() -> impl Pex<'a, Vec<Value>> {
    delimited(tag("<"), spaced_list(simple_value()), tag(">"))
}

fn expr_in_backets<'a>() -> impl Pex<'a, Vec<Symbol>> {
    delimited(tag("("), spaced(Memo::new(expr)), tag(")"))
}

fn dependents(value: Value) -> Vec<Symbol> {
    match value {
        Value::Symbol(s) => [s].into(),
        Value::Reference(s) => [s].into(),
        Value::Expr(ss) => ss,
        Value::Address(_) | Value::Text(_) | Value::Array(_) => Vec::new(),
    }
}

fn flatten(deps: Vec<Symbol>, more_deps: Vec<Vec<Symbol>>) -> Vec<Symbol> {
    if more_deps.is_empty() {
        deps
    } else {
        deps.into_iter()
            .chain(more_deps.into_iter().flat_map(|deps| deps.into_iter()))
            .collect()
    }
}

fn expr<'a>() -> impl Pex<'a, Vec<Symbol>> {
    let operator = tag("||")
        .or(tag("&&"))
        .or(tag("<<"))
        .or(recognize(one_of("|&^*/+-%")));
    let term = || simple_value().map(dependents);
    let binary = preceded(operator, spaced(term()));
    let applic = expr_in_backets();
    let whole = term().and(spaced_list(binary.or(applic)));
    whole.map(|(d, e)| flatten(d, e))
}

fn spacing<'a>() -> impl Pex<'a, ()> {
    value(
        (),
        many0_count(preproc().or(comment()).or(value((), multispace1))),
    )
}

fn preproc<'a>() -> impl Pex<'a, ()> {
    value((), tag("#include").or(tag("# ")).and(not_line_ending))
}

fn comment<'a>() -> impl Pex<'a, ()> {
    value((), tag("/*").and(take_until("*/")).and(tag("*/")))
        .or(value((), tag("//").and(not_line_ending)))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Parser;

    #[test]
    fn prop_test() {
        let mut p = prop();
        let input = "a = b, 12 ;";
        let output = p.parse(input);
        assert_eq!(
            output,
            Ok((
                "",
                Prop {
                    name: Symbol("a".into()),
                    value: [
                        Value::Symbol(Symbol("b".into())),
                        Value::Address(Address(12))
                    ]
                    .into()
                }
            ))
        )
    }

    #[test]
    fn empty_prop_test() {
        let mut p = prop();
        let input = "a ; ";
        let output = p.parse(input);
        assert_eq!(
            output,
            Ok((
                "",
                Prop {
                    name: Symbol("a".into()),
                    value: [].into()
                }
            ))
        )
    }

    #[test]
    fn array_test() {
        let mut p = prop();
        let input = "#size-cells = <0x12 10> ;";
        let output = p.parse(input);
        assert_eq!(
            output,
            Ok((
                "",
                Prop {
                    name: Symbol("#size-cells".into()),
                    value: [Value::Array(
                        [Value::Address(Address(18)), Value::Address(Address(10))].into()
                    )]
                    .into()
                }
            ))
        )
    }
    #[test]
    fn props_test() {
        let mut p = props();
        let input = "a = b, 12 ; x.y = 0x10;";
        let output = p.parse(input);
        assert_eq!(
            output,
            Ok((
                "",
                [
                    Prop {
                        name: Symbol("a".into()),
                        value: [
                            Value::Symbol(Symbol("b".into())),
                            Value::Address(Address(12))
                        ]
                        .into()
                    },
                    Prop {
                        name: Symbol("x.y".into()),
                        value: [Value::Address(Address(16))].into()
                    }
                ]
                .into()
            ))
        )
    }

    #[test]
    fn root_node_test() {
        let mut p = nodes();
        let input = "  /  {} ; ";
        let output = p.parse(input);
        match output {
            Ok(("", nodes)) if nodes.len() == 1 => (),
            wrong => {
                println!("{wrong:?}");
                panic!()
            }
        };
    }

    #[test]
    fn real_node_test() {
        let input = r#"
        / {
            #address-cells = <0x01>;
            #size-cells = <0x01>;
            compatible = "microchip,sama5d27-wlsom1-ek", "microchip,sama5d27-wlsom1", "atmel,sama5d27", "atmel,sama5d2", "atmel,sama5";
            interrupt-parent = <&aic>;
            model = "Microchip SAMA5D27 WLSOM1 EK";
        };
        "#;
        let mut p = nodes();
        let output = p.parse(input);
        match output {
            Ok(("", nodes)) if nodes.len() == 1 => (),
            wrong => {
                println!("{wrong:?}");
                panic!()
            }
        };
    }
    #[test]
    fn comment_preproc_test() {
        let input = r#"
// SPDX-License-Identifier: (GPL-2.0+ OR MIT)
/*
 * at91-sama5d27_wlsom1.dtsi - Device Tree file for SAMA5D27 WLSOM1
 *
 * Copyright (C) 2019 Microchip Technology Inc. and its subsidiaries
 *
 * Author: Nicolas Ferre <nicolas.ferre@microcihp.com>
 * Author: Eugen Hristev <eugen.hristev@microcihp.com>
 */
#include "sama5d2.dtsi"
#include "sama5d2-pinfunc.h"
#include <dt-bindings/gpio/gpio.h>
#include <dt-bindings/mfd/atmel-flexcom.h>
#include <dt-bindings/pinctrl/at91.h>

/ {};

        "#;
        let mut p = tree;
        let output = p.parse(input);
        match output {
            Ok(("", nodes)) if nodes.len() == 1 => (),
            wrong => {
                println!("{wrong:?}");
                panic!()
            }
        };
    }
}
