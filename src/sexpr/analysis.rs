use super::{simplifier::*, Expr};

pub fn trim(x: &Expr) -> Option<Expr> {
    Some(x.as_atom()?.as_string()?.trim().into())
}

pub fn property(name: &str) -> impl Simplifier {
    let name = name.to_owned();

    let key = move |x: &Expr| {
        x.as_atom()?
            .as_string()?
            .eq_ignore_ascii_case(&name)
            .then_some(Expr::key(&name))
    };

    Cons(
        Discard("property"),
        Cons(key, Cons(trim, Discard(Anything))),
    )
}

pub fn footprints() -> impl Simplifier {
    let description = property("description");

    let at = Cons(
        "at",
        Cons(AnyNum, Cons(AnyNum, Or(Cons(AnyNum, Nothing), Nothing))),
    );

    let reference = Cons(
        Discard("fp_text"),
        Cons("reference", Cons(AnyStr, Discard(Anything))),
    );

    let footprint = Cons(
        Discard("footprint"),
        Cons(
            And(AnyStr, LabelAs("library")),
            Filter(Or(reference, Or(at, description))),
        ),
    );

    Cons(Discard("kicad_pcb"), Filter(footprint))
}

pub fn symbols() -> impl Simplifier {
    let ignore_power =
        |x: &Expr| (!x.as_atom()?.as_string()?.starts_with("power:")).then_some(x.clone());

    let lib_id = || Cons("lib_id", Cons(ignore_power, Anything));

    let attribs = Cons("in_bom", Anything)
        .or(Cons("unit", Anything))
        .or(Cons("dnp", Anything))
        .or(lib_id());

    let properties = property("footprint")
        .or(property("reference"))
        .or(property("value"))
        .or(property("MPN"))
        .or(property("manufacturer"))
        .or(property("supply"))
        .or(property("description"));

    let symbol = Cons(
        Discard("symbol"),
        Filter(attribs.or(properties)).and(Ensure(Find(lib_id()))),
    );

    Cons(Discard("kicad_sch"), Filter(symbol))
}

pub fn sheets() -> impl Simplifier {
    let properties = property("sheetname")
        .or(property("sheetfile"))
        .or(property("sheet file"));

    let sheet = Cons(Discard("sheet"), Filter(properties));

    Cons(Discard("kicad_sch"), Filter(sheet))
}
