use kiops::parse_file::write_file;
use kiops::sexpr::parser::parse_expr;

fn main() {
    let path = "data/gateway-arm/gateway.kicad_pcb";
    if let Ok(expr) = kiops::parse_file::parse_file(path, parse_expr) {
        write_file(path, &expr)
    }
}
