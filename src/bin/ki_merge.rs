use kiops::{
    parse_file::{parse_file, parse_stdin, write_stdout, Result},
    sexpr::{parser::parse_s_expr, symlib::merge},
};
use std::env;

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
