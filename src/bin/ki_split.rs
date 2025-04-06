use kiops::{
    parse_file::{parse_stdin, write_file, Result},
    sexpr::{parser::parse_s_expr, symlib::split},
};
use sanitize_filename::sanitize;
use std::env;

fn main() -> Result<()> {
    let usage = "usage: ki_split output_dir";
    let mut args = env::args();
    let _exec = args.next().ok_or(usage)?;
    let output = args.next().ok_or(usage)?;

    let input = parse_stdin(parse_s_expr)?;
    for (name, content) in split(input).ok_or("problem with symbol library contents")? {
        let fname = sanitize(name);
        write_file(&format!("{output}/{fname}.kicad_sym"), &content)?
    }
    Ok(())
}
