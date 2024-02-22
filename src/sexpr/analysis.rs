use super::{simplifier::*, Expr};

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
        "footprint",
        Cons(AnyStr, Filter(Or(reference, Or(at, description)))),
    );

    Cons("kicad_pcb", Filter(footprint))
}

pub fn trim(x: &Expr) -> Option<Expr> {
    Some(x.is_atom()?.string()?.trim().into())
}

pub fn property(name: &str) -> impl Simplifier {
    let name = name.to_owned();

    let key = move |x: &Expr| {
        x.is_atom()?
            .string()?
            .eq_ignore_ascii_case(&name)
            .then_some(Expr::key(&name))
    };

    Cons(
        Discard("property"),
        Cons(key, Cons(trim, Discard(Anything))),
    )
}

pub fn symbols() -> impl Simplifier {
    let properties = Or(property("footprint"), property("reference"));
    let ignore_power =
        |x: &Expr| (!x.is_atom()?.string()?.starts_with("power:")).then_some(x.clone());
    let lib_id = Cons("lib_id", Cons(ignore_power, Anything));
    let symbol = Cons("symbol", Cons(lib_id, Filter(properties)));

    Cons("kicad_sch", Filter(symbol))
}
