use crate::sexpr::simplifier::{Anything, Cons, Discard, Filter, Find, Head, Simplifier};
use crate::sexpr::Expr;
use std::collections::BTreeMap;
use std::iter::once;

const HEADING: &str = "kicad_symbol_lib";
const GENERATOR: &str = "generator";
const VERSION: &str = "version";
const SYMBOL: &str = "symbol";

pub fn merge(input1: Expr, input2: Expr) -> Option<Expr> {
    let version = attr_in(VERSION, &input1)?;
    let generator = attr_in(GENERATOR, &input1)?;
    if version != attr_in(VERSION, &input2)? || generator != attr_in(GENERATOR, &input2)? {
        None?
    }
    let s1 = symbols_in(&input1)?;
    let s2 = symbols_in(&input2)?;
    let symbols = unique(s1.chain(s2));
    let output = symlib(version, generator, symbols);
    Some(output)
}

pub fn split(input: Expr) -> Option<Vec<(String, Expr)>> {
    let symbols = group(symbols_in(&input)?);
    let version = attr_in(VERSION, &input)?;
    let generator = attr_in(GENERATOR, &input)?;
    Some(
        symbols
            .into_iter()
            .map(|(name, symbol)| {
                (
                    name,
                    symlib(version.clone(), generator.clone(), once(symbol)),
                )
            })
            .collect(),
    )
}

pub fn unique(symbols: impl Iterator<Item = Expr>) -> impl Iterator<Item = Expr> {
    group(symbols).into_iter().map(|(_, v)| v)
}

pub fn group(symbols: impl Iterator<Item = Expr>) -> BTreeMap<String, Expr> {
    symbols.filter_map(|s| Some((name_in(&s)?, s))).collect()
}

pub fn attr_in(name: &'static str, symlib: &Expr) -> Option<Expr> {
    Cons(Discard(HEADING), Find(Cons(Discard(name), Head(Anything)))).simplify(symlib)
}

pub fn symbols_in(symlib: &Expr) -> Option<impl Iterator<Item = Expr>> {
    let pattern = Cons(Discard(HEADING), Filter(Cons(SYMBOL, Anything)));
    Some(pattern.simplify(symlib)?.into_deque()?.into_iter())
}

pub fn name_in(symbol: &Expr) -> Option<String> {
    Some(symbol.as_list()?.get(1)?.as_atom()?.as_string()?.to_owned())
}

pub fn list(name: &str, values: impl Iterator<Item = Expr>) -> Expr {
    Expr::list(once(Expr::key(name)).chain(values))
}

pub fn attr(name: &str, value: Expr) -> impl Iterator<Item = Expr> {
    once(list(name, once(value)))
}

pub fn symlib(version: Expr, generator: Expr, symbols: impl Iterator<Item = Expr>) -> Expr {
    list(
        HEADING,
        attr(VERSION, version)
            .chain(attr(GENERATOR, generator))
            .chain(symbols),
    )
}
