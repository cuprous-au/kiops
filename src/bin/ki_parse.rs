use kiops::parse_file::{parse_stdin, write_stdout, Result};
use kiops::sexpr::analysis::{footprints, sheets, symbols};
use kiops::sexpr::json::expr_to_json_value;
use kiops::sexpr::parser::parse_s_expr;
use kiops::sexpr::simplifier::{Anything, Simplifier};
use kiops::sexpr::Expr;
use std::env;

fn main() -> Result<()> {
    let usage = "ki_parse: command [-s]";
    let mut args = env::args();
    let _exec = args.next().ok_or(usage)?;
    let command = args.next().ok_or(usage)?;
    let want_sexpr = args.next().is_some_and(|s| s == "-s");

    let output = match &*command {
        "footprints" => run(footprints())?,
        "symbols" => run(symbols())?,
        "sheets" => run(sheets())?,
        "format" => run(Anything)?,
        _ => Err("argument not recognised")?,
    };

    if want_sexpr {
        write_stdout(&output)?;
    } else {
        let json = expr_to_json_value(output);
        write_stdout(&json)?;
    }

    Ok(())
}

fn run(simplifier: impl Simplifier) -> Result<Expr> {
    let input = parse_stdin(parse_s_expr)?;

    Ok(simplifier
        .simplify(&input)
        .ok_or("unrecognised file contents")?)
}
