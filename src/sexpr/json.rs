use super::{Atom, Expr};
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

/// Convert an s-expression to a JSON value.
/// A list beginning with symbol becomes a JSON property.
/// The following items contribute to its value.
/// Property items in a list are gathered into a JSON object.
/// Other items are gathered into a JSON array.
pub fn expr_to_json_value(expr: Expr) -> Value {
    match particle(expr) {
        Property(s, v) => {
            let mut obj = Map::new();
            obj.insert(s, v);
            Value::Object(obj)
        }
        Free(v) => v,
    }
}

/// An `Expr` can be converted to a `Particle`.
/// A List beginning with a Symbol becomes a Property particle.
/// Other values become a Free particles.
#[derive(Debug)]
enum Particle {
    Property(String, Value),
    Free(Value),
}

use Particle::*;

fn particle(expr: Expr) -> Particle {
    match expr {
        Expr::Constant(a) => match a {
            Atom::Symbol(v) => Free(Value::String(v)),
            Atom::Bits(v) => Free(Value::Number(v.into())),
            Atom::Bool(v) => Free(Value::Bool(v)),
            Atom::Str(v) => Free(Value::String(v)),
            Atom::Num(v) => Number::from_f64(v)
                .map(|x| Free(Value::Number(x)))
                .unwrap_or(Free(Value::Null)),
            Atom::Uuid(v) => Free(Value::String(v.to_string())),
        },

        Expr::List(xs) => match xs.front() {
            Some(Expr::Constant(Atom::Symbol(s))) => {
                Property(s.to_owned(), gather(xs.into_iter().skip(1)))
            }
            _ => Free(gather(xs.into_iter())),
        },
    }
}

fn gather(exprs: impl Iterator<Item = Expr>) -> Value {
    let mut map = BTreeMap::<String, Vec<Value>>::new();
    let mut free = Vec::<Value>::new();

    // each expr becomes a json particle
    for x in exprs {
        let p = particle(x);

        match p {
            // a property's value goes into a multimap
            Property(s, v) => map.entry(s).or_default().push(v),
            // a free value goes into a vector
            Free(v) => free.push(v),
        }
    }

    // the multimap is converted to a json map
    let mut obj = Map::new();
    for (s, mut vs) in map {
        if vs.len() == 1 {
            // singleton properties are unwrapped
            obj.insert(s, vs.pop().unwrap());
        } else {
            // multiple valued properties become json arrays
            obj.insert(s, Value::Array(vs));
        }
    }

    // classify the collected particles as a single json value, json array or object
    match obj.len() {
        0 => match free.len() {
            1 => free.pop().unwrap(),
            _ => Value::Array(free),
        },
        _ => match free.len() {
            0 => Value::Object(obj),
            _ => {
                free.push(Value::Object(obj));
                Value::Array(free)
            }
        },
    }
}
