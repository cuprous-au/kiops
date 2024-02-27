use kiops::{
    dts::{
        analysis::{Analysis, LabelView, Xref},
        parser::tree,
        Node,
    },
    parse_file::{parse_file, write_file, Result},
};

fn main() -> Result<()> {
    let mut all_nodes: Vec<Node> = Vec::new();

    for fp in [
        "data/linux/arch/arm/boot/dts/sama5d2.dtsi",
        "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1.dtsi",
        "data/linux/arch/arm/boot/dts/at91-sama5d27_wlsom1_ek.dts",
    ] {
        let nodes = parse_file(fp, tree)?;
        all_nodes.extend(nodes)
    }

    let analysis = Analysis::new(all_nodes);
    write_file("results/sama5d27-analysis.log", &analysis)?;

    let xref = Xref::new(&analysis, |s| s.starts_with("PIN_"));
    write_file("results/sama5d27-xref.log", &xref)?;

    let lview = LabelView::new(&analysis);
    write_file("results/sama5d27-labels.md", &lview)?;

    Ok(())
}
