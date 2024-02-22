use std::io::stdout;

fn main() {
    if let Ok(nodes) = kiops::parse_file::parse_stdin(kiops::dts::parser::tree) {
        serde_json::to_writer(stdout(), &nodes).expect("could not serialize to JSON")
    }
}
