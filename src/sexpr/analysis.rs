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

pub fn any_property() -> impl Simplifier {
    let string_as_symbol = |expr: &Expr| Some(Expr::key(expr.as_atom()?.as_string()?));

    Cons(
        Discard("property"),
        Cons(string_as_symbol, Cons(trim, Discard(Anything))),
    )
}

pub fn footprints() -> impl Simplifier {
    let description = property("description");

    let at = Cons(
        "at",
        Cons(AnyNum, Cons(AnyNum, Or(Cons(AnyNum, Nothing), Nothing))),
    );

    let model = Cons("model", Cons(AnyStr, Discard(Anything)));

    let reference = Cons(
        Discard("fp_text"),
        Cons("reference", Cons(AnyStr, Discard(Anything))),
    );

    let footprint = {
        Cons(
            Discard("footprint"),
            Cons(
                And(AnyStr, LabelAs("library")),
                Filter(reference.or(at).or(description).or(model)),
            ),
        )
    };

    let enlist = |expr: &Expr| -> Option<Expr> { Some(Expr::list([expr.clone()])) };

    Cons(Discard("kicad_pcb"), Filter(footprint.clone())).or(footprint.clone().and(enlist))
}

pub fn symbols() -> impl Simplifier {
    let is_power_id = |x: &Expr| {
        x.as_atom()?
            .as_string()?
            .starts_with("power:")
            .then_some(x.clone())
    };

    let is_power_symbol = Cons("lib_id", Cons(is_power_id, Nothing));

    let attribs = Cons("in_bom", Anything)
        .or(Cons("unit", Anything))
        .or(Cons("dnp", Anything))
        .or(Cons("uuid", Anything))
        .or(Cons("lib_id", Anything));

    let symbol = Cons(
        Discard("symbol"),
        Filter(attribs.or(any_property())).and(Not(Find(is_power_symbol))),
    );

    Cons(Discard("kicad_sch".or("kicad_symbol_lib")), Filter(symbol))
}

pub fn sheets() -> impl Simplifier {
    let properties = property("sheetname")
        .or(property("sheetfile"))
        .or(property("sheet file"));

    let sheet = Cons(Discard("sheet"), Filter(properties));

    Cons(Discard("kicad_sch"), Filter(sheet))
}
