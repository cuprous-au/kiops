use std::fmt::Display;

use serde::Serialize;

pub mod analysis;
pub mod parser;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Address(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum NodeName {
    Symbol(Symbol, Option<Address>),
    Reference(Symbol),
}

impl NodeName {
    pub fn is_root(&self) -> bool {
        matches!(self, NodeName::Symbol(Symbol(s), None) if s == "")
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, NodeName::Reference(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Value {
    Symbol(Symbol),
    Reference(Symbol),
    Address(Address),
    Text(String),
    Array(Vec<Value>),
    Expr(Vec<Symbol>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Prop {
    pub name: Symbol,
    pub value: Vec<Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub labels: Vec<Symbol>,
    pub name: NodeName,
    pub props: Vec<Prop>,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(Vec<NodeName>);

impl Path {
    pub fn root() -> Self {
        Self([NodeName::Symbol(Symbol("".into()), None)].into())
    }

    pub fn reference(label: &Symbol) -> Self {
        Self([NodeName::Reference(label.clone())].into())
    }

    pub fn join(mut self, relpath: impl IntoIterator<Item = NodeName>) -> Path {
        for name in relpath {
            if name.is_reference() || name.is_root() {
                self.0 = Vec::new()
            }
            self.0.push(name)
        }
        self
    }

    pub fn is_root(&self) -> bool {
        self.0.len() == 1 && self.0[0].is_root()
    }

    pub fn is_reference(&self) -> bool {
        self.0.len() > 0 && self.0[0].is_reference()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn split(&self) -> (NodeName, Vec<NodeName>) {
        let mut path = if self.0.is_empty() {
            Path::root().0
        } else {
            self.0.clone()
        };
        let first = path.remove(0);
        (first, path)
    }

    pub fn parent(&self) -> Self {
        let mut parent = self.clone();
        if parent.0.len() > 1 {
            parent.0.pop();
        }
        parent
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("0x{:x}", self.0))
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeName::Symbol(s, None) => s.fmt(f),
            NodeName::Symbol(s, Some(a)) => f.write_fmt(format_args!("{s}@{a}")),
            NodeName::Reference(s) => f.write_fmt(format_args!("&{s}")),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_root() {
            f.write_str("/")
        } else {
            fmt_delimited(&self.0, "", "/", "", f)
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Symbol(s) => s.fmt(f),
            Value::Reference(s) => f.write_fmt(format_args!("&{s}")),
            Value::Address(a) => a.fmt(f),
            Value::Text(t) => f.write_fmt(format_args!("\"{t}\"")),
            Value::Array(a) => fmt_delimited(a, "<", " ", ">", f),
            Value::Expr(r) => fmt_delimited(r, "(", ", ", ")", f),
        }
    }
}

impl Display for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        f.write_str(" = ")?;
        match self.value.len() {
            0 => f.write_str("true")?,
            1 => self.value[0].fmt(f)?,
            _ => fmt_delimited(&self.value, "[", ", ", "]", f)?,
        }
        Ok(())
    }
}

fn fmt_delimited<A>(
    elems: &[A],
    open: &str,
    sep: &str,
    close: &str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result
where
    A: Display,
{
    f.write_str(open)?;
    let mut elems = elems.iter();
    if let Some(head) = elems.next() {
        head.fmt(f)?;
        for elem in elems {
            f.write_str(sep)?;
            elem.fmt(f)?;
        }
    }
    f.write_str(close)?;
    Ok(())
}
