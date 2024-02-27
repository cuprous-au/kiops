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
    let properties = Or(property("footprint"), property("reference"));
    let ignore_power =
        |x: &Expr| (!x.as_atom()?.as_string()?.starts_with("power:")).then_some(x.clone());
    let lib_id = Cons("lib_id", Cons(ignore_power, Anything));
    let symbol = Cons(Discard("symbol"), Cons(lib_id, Filter(properties)));

    Cons(Discard("kicad_sch"), Filter(symbol))
}

pub fn sheets() -> impl Simplifier {
    let properties = Or(
        property("sheetname"),
        Or(property("sheetfile"), property("sheet file")),
    );
    let sheet = Cons(Discard("sheet"), Filter(properties));
    Cons(Discard("kicad_sch"), Filter(sheet))
}
