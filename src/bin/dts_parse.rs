use kiops::{
    dts::parser::tree,
    parse_file::{parse_stdin, Result},
};
use std::io::stdout;

fn main() -> Result<()> {
    let nodes = parse_stdin(tree)?;
    serde_json::to_writer(stdout(), &nodes)?;
    Ok(())
}
