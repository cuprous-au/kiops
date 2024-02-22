pub mod analysis;
pub mod parser;
pub mod simplifier;

use std::{collections::VecDeque, fmt::Display};
use uuid::Uuid;

/// Indivisible values in an S-expression
#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Symbol(String),
    Str(String),
    Num(f64),
    Bits(u64),
    Bool(bool),
    Uuid(Uuid),
}

impl Atom {
    pub fn string(&self) -> Option<&str> {
        match self {
            Atom::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn symbol(&self) -> Option<&str> {
        match self {
            Atom::Symbol(s) => Some(s),
            _ => None,
        }
    }
    pub fn num(&self) -> Option<f64> {
        match self {
            Atom::Num(n) => Some(*n),
            _ => None,
        }
    }
    pub fn bits(&self) -> Option<u64> {
        match self {
            Atom::Bits(n) => Some(*n),
            _ => None,
        }
    }
    pub fn boolean(&self) -> Option<bool> {
        match self {
            Atom::Bool(n) => Some(*n),
            _ => None,
        }
    }
    pub fn uuid(&self) -> Option<Uuid> {
        match self {
            Atom::Uuid(n) => Some(*n),
            _ => None,
        }
    }
}

impl From<&str> for Atom {
    fn from(value: &str) -> Self {
        Atom::Str(value.to_owned())
    }
}

impl From<String> for Atom {
    fn from(value: String) -> Self {
        Atom::Str(value)
    }
}

impl From<f64> for Atom {
    fn from(value: f64) -> Self {
        Atom::Num(value)
    }
}

impl From<u64> for Atom {
    fn from(value: u64) -> Self {
        Atom::Bits(value)
    }
}

impl From<bool> for Atom {
    fn from(value: bool) -> Self {
        Atom::Bool(value)
    }
}

impl From<Uuid> for Atom {
    fn from(value: Uuid) -> Self {
        Atom::Uuid(value)
    }
}

/// An S-expression
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Constant(Atom),
    List(VecDeque<Expr>),
}

impl Expr {
    pub fn key(name: &str) -> Expr {
        Expr::Constant(Atom::Symbol(name.to_string()))
    }

    pub fn list(values: impl IntoIterator<Item = Expr>) -> Expr {
        Expr::List(values.into_iter().collect())
    }

    pub fn empty() -> Expr {
        Expr::List(VecDeque::new())
    }

    pub fn is_atom(&self) -> Option<&Atom> {
        match self {
            Expr::Constant(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_list(&self) -> Option<&VecDeque<Expr>> {
        match self {
            Expr::List(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Expr::List(value) => value.is_empty(),
            _ => false,
        }
    }

    pub fn into_deque(self) -> Option<VecDeque<Expr>> {
        match self {
            Expr::List(elems) => Some(elems),
            _ => None,
        }
    }
}

impl<A> From<A> for Expr
where
    A: Into<Atom>,
{
    fn from(value: A) -> Self {
        Expr::Constant(value.into())
    }
}

impl Default for Expr {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Symbol(v) => v.fmt(f),
            Atom::Str(v) => {
                "\"".fmt(f)?;
                v.replace('"', "\\\"").fmt(f)?;
                "\"".fmt(f)
            }
            Atom::Num(v) => v.fmt(f),
            Atom::Bits(v) => f.write_fmt(format_args!("0x{v:x}")),
            Atom::Bool(v) => v.fmt(f),
            Atom::Uuid(v) => v.fmt(f),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_expr(self, 0, f)
    }
}

fn fmt_expr(expr: &Expr, indent: usize, target: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match expr {
        Expr::Constant(atom) => atom.fmt(target)?,
        Expr::List(exprs) => {
            "(".fmt(target)?;
            let mut iter = exprs.iter();
            if let Some(expr) = iter.next() {
                fmt_expr(expr, indent, target)?;
                let n = exprs.len();
                if n <= 4 {
                    for expr in iter {
                        " ".fmt(target)?;
                        fmt_expr(expr, indent, target)?
                    }
                } else {
                    let indent = indent + 2;
                    for expr in iter {
                        "\n".fmt(target)?;
                        for _ in 0..indent {
                            " ".fmt(target)?
                        }
                        fmt_expr(expr, indent, target)?
                    }
                }
            }
            ")".fmt(target)?;
        }
    }
    Ok(())
}
