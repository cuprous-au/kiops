use kiops::{
    parse_file::{parse_stdin, read_json, write_stdout, Result},
    sexpr::{
        edit::{editor, extract_props},
        parser::parse_s_expr,
        simplifier::Simplifier,
    },
};
use std::env;

fn main() -> Result<()> {
    let usage = "usage: ki_edit symbol_props_file";
    let mut args = env::args();
    let _exec = args.next().ok_or(usage)?;
    let fname = args.next().ok_or(usage)?;

    let json = read_json(&fname)?;
    let props = extract_props(json).ok_or("invalid symbol properties")?;
    let simplifier = editor(props);

    let input = parse_stdin(parse_s_expr)?;
    let output = simplifier
        .simplify(&input)
        .ok_or("unrecognised input file contents")?;

    write_stdout(&output)?;
    Ok(())
}
