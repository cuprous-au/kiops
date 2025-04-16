use super::{
    simplifier::{Anything, Cons, Discard, Filter, Find, Head, Simplifier},
    Atom, Expr,
};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, rc::Rc};

/// The name of the special reference property
const REFERENCE: &str = "Reference";

/// The type of a key in the map of properties
/// consisting of a symbol name and a property name
pub type Key = (String, String);

/// Interpret a serde json `Value` as an array of objects and reform this as a `BTreeMap`.
/// Each member of each object becomes an entry in the map.
/// Each object has a string-valued member called `REFERENCE`
/// which becomes the first part of the key.
pub fn extract_props(json: Value) -> Option<BTreeMap<Key, Atom>> {
    json.as_array().map(|records| {
        let records = records.iter().filter_map(|r| {
            let record = r.as_object()?.clone();
            let reference = record.get(REFERENCE)?.as_str()?.to_string();
            Some((reference, record))
        });

        let to_key_value = |reference, property, value: Value| {
            let key = (reference, property);

            let value = value
                .as_f64()
                .map(Atom::from)
                .or(value.as_str().map(Atom::from))?;

            Some((key, value))
        };

        let to_key_values = |(reference, record): (String, Map<String, Value>)| {
            record
                .into_iter()
                .filter(|(property, _)| property != REFERENCE)
                .filter_map(move |(property, value)| {
                    to_key_value(reference.clone(), property, value)
                })
        };

        records.flat_map(to_key_values).collect()
    })
}

pub fn editor(props: BTreeMap<Key, Atom>) -> impl Simplifier {
    let props = Rc::new(props);

    let property_body = move |sym_name: String| {
        let props = props.clone();

        move |expr: &Expr| {
            let elems = expr.as_list()?;
            let prop_name = elems.front()?.as_atom()?.as_string()?.to_string();
            let key = (sym_name.clone(), prop_name);
            let value = props.get(&key)?.clone();
            let mut elems = elems.clone();
            *elems.get_mut(1)? = value.into();
            Some(Expr::List(elems))
        }
    };

    let property = move |sym_name: String| Cons("property", property_body(sym_name).or(Anything));

    let reference = Find(Cons(
        Discard("property"),
        Cons(Discard(Atom::from(REFERENCE)), Head(Anything)),
    ));

    let symbol_body = move |expr: &Expr| {
        let sym_name = reference
            .simplify(expr)?
            .as_atom()?
            .as_string()?
            .to_string();

        Filter(property(sym_name).or(Anything)).simplify(expr)
    };

    let symbol = Cons("symbol", symbol_body);

    Cons("kicad_sch", Filter(symbol.or(Anything)))
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::{
        parse_file::parse_with,
        sexpr::{parser::parse_s_expr, simplifier::Simplifier, Atom, Expr},
    };

    use super::{editor, extract_props, Key};

    #[test]
    fn test_extract() {
        let value = json!([
            { "Reference": "a", "Footprint": "b" },
            { "Reference": "c", "Footprint": "d" },
            {},
            { "Footprint": "e" }
        ]);

        let m = extract_props(value).unwrap();

        assert_eq!(
            m.get(&("a".to_string(), "Footprint".to_string())).unwrap(),
            &Atom::from("b")
        );

        assert_eq!(
            m.get(&("c".to_string(), "Footprint".to_string())).unwrap(),
            &Atom::from("d")
        );
    }

    fn schematic_before() -> Expr {
        let s = r#"
            (kicad_sch
                (version 20250114)
                (generator "eeschema")
                (generator_version "9.0")
                (uuid "e63e39d7-6ac0-4ffd-8aa3-1841a4541b55")
                (paper "A4")
                (title_block
                    (title "Cuprous Secured Edge Gateway v4")
                    (date "2025-04-09")
                    (rev "1")
                    (company "Copyright © 2025 Cuprous Pty Ltd Australia")
                    (comment 1 "cuprous.com.au")
                )
                (lib_symbols
                    (symbol "Cuprous:3220-10-0300-00"
                        (pin_names
                            (offset 1.016)
                        )
                        (exclude_from_sim no)
                        (in_bom yes)
                        (on_board yes)
                        (property "Reference" "J"
                            (at -8.89 8.255 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify left bottom)
                            )
                        )
                        (property "Value" "3220-10-0300-00"
                            (at -8.89 -10.16 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify left bottom)
                            )
                        )
                        (property "Footprint" "3220-10-0300-00:CNC_3220-10-0300-00"
                            (at 0 0 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify bottom)
                                (hide yes)
                            )
                        )
                    )
                )
                (symbol
                    (lib_id "Cuprous:3220-10-0300-00")
                    (at 53.34 134.62 0)
                    (unit 1)
                    (exclude_from_sim no)
                    (in_bom yes)
                    (on_board yes)
                    (dnp no)
                    (fields_autoplaced yes)
                    (uuid "11cc29a1-afce-483d-83cc-2917e2a927e5")
                    (property "Reference" "J8"
                        (at 53.34 121.92 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                        )
                    )
                    (property "Value" "Panel"
                        (at 53.34 124.46 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                        )
                    )
                    (property "Footprint" "Cuprous:CNC_3220-10-0300-00"
                        (at 53.34 134.62 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                            (justify bottom)
                            (hide yes)
                        )
                    )
                    (property "Datasheet" ""
                        (at 53.34 134.62 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                            (hide yes)
                        )
                    )
                )
                (embedded_fonts no)
            )
            "#;
        parse_with(s, parse_s_expr).unwrap()
    }

    fn schematic_after() -> Expr {
        let s = r#"
            (kicad_sch
                (version 20250114)
                (generator "eeschema")
                (generator_version "9.0")
                (uuid "e63e39d7-6ac0-4ffd-8aa3-1841a4541b55")
                (paper "A4")
                (title_block
                    (title "Cuprous Secured Edge Gateway v4")
                    (date "2025-04-09")
                    (rev "1")
                    (company "Copyright © 2025 Cuprous Pty Ltd Australia")
                    (comment 1 "cuprous.com.au")
                )
                (lib_symbols
                    (symbol "Cuprous:3220-10-0300-00"
                        (pin_names
                            (offset 1.016)
                        )
                        (exclude_from_sim no)
                        (in_bom yes)
                        (on_board yes)
                        (property "Reference" "J"
                            (at -8.89 8.255 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify left bottom)
                            )
                        )
                        (property "Value" "3220-10-0300-00"
                            (at -8.89 -10.16 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify left bottom)
                            )
                        )
                        (property "Footprint" "3220-10-0300-00:CNC_3220-10-0300-00"
                            (at 0 0 0)
                            (effects
                                (font
                                    (size 1.27 1.27)
                                )
                                (justify bottom)
                                (hide yes)
                            )
                        )
                    )
                )
                (symbol
                    (lib_id "Cuprous:3220-10-0300-00")
                    (at 53.34 134.62 0)
                    (unit 1)
                    (exclude_from_sim no)
                    (in_bom yes)
                    (on_board yes)
                    (dnp no)
                    (fields_autoplaced yes)
                    (uuid "11cc29a1-afce-483d-83cc-2917e2a927e5")
                    (property "Reference" "J8"
                        (at 53.34 121.92 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                        )
                    )
                    (property "Value" "Panel"
                        (at 53.34 124.46 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                        )
                    )
                    (property "Footprint" "Cuprous:other_footprint"
                        (at 53.34 134.62 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                            (justify bottom)
                            (hide yes)
                        )
                    )
                    (property "Datasheet" ""
                        (at 53.34 134.62 0)
                        (effects
                            (font
                                (size 1.27 1.27)
                            )
                            (hide yes)
                        )
                    )
                )
                (embedded_fonts no)
            )
            "#;
        parse_with(s, parse_s_expr).unwrap()
    }

    #[test]
    fn test_no_edits() {
        let m: BTreeMap<Key, Atom> = [(("".to_string(), "".to_string()), Atom::from(""))]
            .into_iter()
            .collect();
        let s = editor(m).simplify(&schematic_before()).unwrap();
        assert_eq!(s, schematic_before());
    }

    #[test]
    fn test_edits() {
        let m: BTreeMap<Key, Atom> = [(
            ("J8".to_string(), "Footprint".to_string()),
            Atom::from("Cuprous:other_footprint"),
        )]
        .into_iter()
        .collect();
        let s = editor(m).simplify(&schematic_before()).unwrap();
        assert_eq!(s, schematic_after());
    }
}
