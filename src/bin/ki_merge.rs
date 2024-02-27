use kiops::parse_file::{parse_file, parse_stdin, write_stdout, Result};
use kiops::sexpr::parser::parse_s_expr;
use kiops::sexpr::simplifier::{Anything, Cons, Discard, Filter, Find, Head, Simplifier};
use kiops::sexpr::Expr;
use std::collections::HashMap;
use std::env;
use std::iter::once;

const HEADING: &str = "kicad_symbol_lib";
const GENERATOR: &str = "generator";
const VERSION: &str = "version";
const SYMBOL: &str = "symbol";

fn main() -> Result<()> {
    let usage = "usage: ki_merge input";
    let mut args = env::args();
    let _exec = args.next().ok_or(usage)?;
    let path = args.next().ok_or(usage)?;

    let input1 = parse_stdin(parse_s_expr)?;
    let input2 = parse_file(&path, parse_s_expr)?;
    let output = merge(input1, input2).ok_or("library version missmatch")?;

    write_stdout(&output)?;
    Ok(())
}

fn merge(input1: Expr, input2: Expr) -> Option<Expr> {
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

fn unique(symbols: impl Iterator<Item = Expr>) -> impl Iterator<Item = Expr> {
    let map: HashMap<String, Expr> = symbols.filter_map(|s| Some((name_in(&s)?, s))).collect();
    map.into_iter().map(|(_, v)| v)
}

fn attr_in(name: &'static str, symlib: &Expr) -> Option<Expr> {
    Cons(Discard(HEADING), Find(Cons(Discard(name), Head(Anything)))).simplify(symlib)
}

fn symbols_in(symlib: &Expr) -> Option<impl Iterator<Item = Expr>> {
    let pattern = Cons(Discard(HEADING), Filter(Cons(SYMBOL, Anything)));

    Some(pattern.simplify(symlib)?.into_deque()?.into_iter())
}

fn name_in(symbol: &Expr) -> Option<String> {
    Some(symbol.as_list()?.get(1)?.as_atom()?.as_string()?.to_owned())
}

fn list(name: &str, values: impl Iterator<Item = Expr>) -> Expr {
    Expr::list(once(Expr::key(name)).chain(values))
}

fn attr(name: &str, value: Expr) -> impl Iterator<Item = Expr> {
    once(list(name, once(value)))
}

fn symlib(version: Expr, generator: Expr, symbols: impl Iterator<Item = Expr>) -> Expr {
    list(
        HEADING,
        attr(VERSION, version)
            .chain(attr(GENERATOR, generator))
            .chain(symbols),
    )
}
