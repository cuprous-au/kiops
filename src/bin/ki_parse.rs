use std::env;

use kiops::parse_file::{parse_stdin, write_stdout};
use kiops::sexpr::analysis::{footprints, symbols};
use kiops::sexpr::parser::parse_expr;
use kiops::sexpr::simplifier::Simplifier;

fn main() {
    let command = env::args().skip(1).next().expect("argument required");
    let simplifier: Box<dyn Simplifier> = match &*command {
        "footprints" => Box::new(footprints()),
        "symbols" => Box::new(symbols()),
        _ => None.expect("argument not recognised"),
    };
    let expr = parse_stdin(parse_expr).expect("parsing error");
    let result = simplifier
        .simplify(&expr)
        .expect("unexpected file contents");
    write_stdout(&result)
}
