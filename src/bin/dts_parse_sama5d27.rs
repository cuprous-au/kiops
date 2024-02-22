use kiops::dts::{
    analysis::{Analysis, LabelView, Xref},
    Node,
};
use std::fs;

fn main() {
    let mut all_nodes: Vec<Node> = Vec::new();

    for fp in [
        "data/linux/arch/arm/boot/dts/sama5d2.dtsi",
        "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1.dtsi",
        "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1_ek.dts",
    ] {
        if let Ok(nodes) = kiops::parse_file::parse_file(fp, kiops::dts::parser::tree) {
            all_nodes.extend(nodes)
        }
    }

    let analysis = Analysis::new(all_nodes);
    fs::write("results/sama5d27-analysis.log", analysis.to_string())
        .expect("Could not create output file");

    let xref = Xref::new(&analysis, |s| s.starts_with("PIN_"));
    fs::write("results/sama5d27-xref.log", xref.to_string()).expect("Could not create output file");

    let lview = LabelView::new(&analysis);
    fs::write("results/sama5d27-labels.md", lview.to_string())
        .expect("Could not create output file");
}
