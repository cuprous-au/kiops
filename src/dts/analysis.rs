use std::{
    collections::BTreeMap,
    fmt::Display,
    mem::take,
    ops::Bound::{Excluded, Unbounded},
};

use super::*;

/// Path names, labels, dependents and symbols for a set of nodes.
#[derive(Default, Debug)]
pub struct Analysis {
    paths: BTreeMap<Path, Node>,
    labels: BTreeMap<Symbol, Path>,
    reverse: BTreeMap<Path, Vec<Path>>,
    symbols: BTreeMap<Symbol, Vec<Path>>,
}

impl Analysis {
    /// Create a basic analysis of a series of dts nodes.
    pub fn new(nodes: impl IntoIterator<Item = Node>) -> Self {
        let mut analysis = Self::default();
        analysis.gather(&Path::root(), nodes);
        analysis.merge();
        analysis.index();
        analysis.index_symbols();
        analysis
    }

    /// Gather a collection of nodes into the analysis data structure.
    /// Each node is registered under a path name which begins with
    /// either the root or a label reference.
    fn gather(&mut self, parent: &Path, nodes: impl IntoIterator<Item = Node>) {
        for mut node in nodes {
            // the pathname for this node
            let path = parent.clone().join([node.name.clone()]);

            // remove the children
            let nodes = take(&mut node.nodes);

            // catalogue the labels on this path
            for label in node.labels.iter().cloned() {
                if let Some(conflict) = self.labels.insert(label.clone(), path.clone()) {
                    if self.absolute(conflict.clone()) != self.absolute(path.clone()) {
                        println!("Conflict: label {label} refers to {path} and {conflict}")
                    }
                }
            }

            // catalogue the path and node
            self.upsert(path.clone(), node);

            // gather the children
            self.gather(&path, nodes);
        }
    }

    /// Merge the label references with labelled nodes.
    /// In this process label references are eliminated.
    fn merge(&mut self) {
        let references: Vec<Path> = self
            .paths
            .keys()
            .filter(|p| p.is_reference())
            .cloned()
            .collect();

        for path in references {
            if let Some(node) = self.paths.remove(&path) {
                self.upsert(self.absolute(path), node);
            }
        }
    }

    /// Build the reverse index, from node to directly dependent nodes.
    fn index(&mut self) {
        for (path1, node) in self.paths.iter() {
            for refer in refs_in_node(&node) {
                let path2 = self.absolute(Path::reference(&refer));
                self.reverse
                    .entry(path2.clone())
                    .or_default()
                    .push(path1.clone());
            }
        }
    }

    /// Build the symbol index from symbols to nodes that use them
    fn index_symbols(&mut self) {
        for (path, node) in self.paths.iter() {
            for symbol in symbols_in_node(node) {
                self.symbols.entry(symbol).or_default().push(path.clone());
            }
        }
    }

    /// Insert a single node or merge the node with an existing node with the same path name.
    fn upsert(&mut self, key: Path, mut node: Node) {
        let merge = |extant: &mut Node| {
            extant.labels.extend(take(&mut node.labels));
            extant.props.extend(take(&mut node.props))
        };
        self.paths.entry(key).and_modify(merge).or_insert(node);
    }

    /// Reference to the node at a path
    pub fn node_at(&self, path: &Path) -> Option<&Node> {
        self.paths.get(&self.absolute(path.clone()))
    }

    /// Iterator over children of a node
    pub fn children(&self, path: &Path) -> impl Iterator<Item = (&Path, &Node)> {
        let base = path.len();
        self.paths
            .contains_key(path)
            .then(|| {
                self.paths
                    .range((Excluded(path), Unbounded))
                    .take_while(move |(k, _)| k.len() > base)
            })
            .into_iter()
            .flatten()
    }

    /// Compute all dependents for a node, recursively.
    /// Result has deepest dependents first ie:
    /// Any n+1th generation dependent appears before
    /// any nth generation dependent.
    pub fn dependents(&self, start: &Path) -> Vec<Path> {
        self.reverse
            .get(start)
            .into_iter()
            .flat_map(|direct_deps| {
                direct_deps
                    .iter()
                    .flat_map(|direct_dep| self.dependents(direct_dep))
                    .chain(direct_deps.iter().cloned())
            })
            .collect()
    }

    /// Translate a path which may start with a reference
    /// to an absolute path starting at the root.
    /// References may refer to references so this is recursive.
    pub fn absolute(&self, path: Path) -> Path {
        if let (NodeName::Reference(label), relpath) = path.split() {
            if let Some(refer) = self.labels.get(&label) {
                self.absolute(refer.clone().join(relpath))
            } else {
                println!("Undefined label: {label}");
                path
            }
        } else {
            path
        }
    }
}

/// A label oriented view of an Analysis
pub struct LabelView<'a>(&'a Analysis);

impl<'a> LabelView<'a> {
    pub fn new<'b>(analysis: &'b Analysis) -> Self
    where
        'b: 'a,
    {
        Self(analysis)
    }
}

/// A cross reference from symbols to nodes.
pub struct Xref(pub Vec<SymbolAssoc>);

impl Xref {
    /// Build a cross reference considering only symbols
    /// whose names that match a predicate.
    pub fn new(analysis: &Analysis, pred: impl Fn(&str) -> bool) -> Self {
        let assoc = |(path, node): (&Path, &Node)| {
            let mut symbols: Vec<Symbol> = symbols_in_node(node)
                .into_iter()
                .filter(|s| pred(&s.0))
                .collect();

            if !symbols.is_empty() {
                symbols.sort();
                symbols.dedup();

                Some(SymbolAssoc {
                    first: symbols.remove(0),
                    others: symbols,
                    path: path.clone(),
                    node: node.clone(),
                })
            } else {
                None
            }
        };

        let mut assocs: Vec<SymbolAssoc> = analysis.paths.iter().filter_map(assoc).collect();
        assocs.sort_by_key(|a| a.first.clone());
        Self(assocs)
    }
}

pub struct SymbolAssoc {
    pub first: Symbol,
    pub others: Vec<Symbol>,
    pub path: Path,
    pub node: Node,
}

fn symbols_in_node(node: &Node) -> Vec<Symbol> {
    fn search_value(value: &Value) -> Vec<Symbol> {
        match value {
            Value::Symbol(s) => [s.clone()].into(),
            Value::Array(a) => a.iter().flat_map(search_value).collect(),
            Value::Expr(r) => r.clone(),
            Value::Reference(_) | Value::Address(_) | Value::Text(_) => Vec::new(),
        }
    }

    fn search_prop(prop: &Prop) -> Vec<Symbol> {
        prop.value.iter().flat_map(search_value).collect()
    }

    node.props.iter().flat_map(search_prop).collect()
}

fn refs_in_node(node: &Node) -> Vec<Symbol> {
    fn search_value(value: &Value) -> Vec<Symbol> {
        match value {
            Value::Reference(s) => [s.clone()].into(),
            Value::Array(a) => a.iter().flat_map(search_value).collect(),
            Value::Symbol(_) | Value::Expr(_) | Value::Address(_) | Value::Text(_) => Vec::new(),
        }
    }

    fn search_prop(prop: &Prop) -> Vec<Symbol> {
        prop.value.iter().flat_map(search_value).collect()
    }

    node.props.iter().flat_map(search_prop).collect()
}

impl Display for Analysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Nodes\n")?;
        for (path, node) in self.paths.iter() {
            fmt_path_node("", path, node, f)?
        }
        f.write_str("Labels\n")?;
        for (label, path) in self.labels.iter() {
            f.write_fmt(format_args!("{label}: {path}\n"))?;
        }
        f.write_str("Dependents\n")?;
        for path in self.paths.keys() {
            if let Some(deps) = self.reverse.get(path) {
                f.write_fmt(format_args!("{path}\n"))?;
                for dep in deps {
                    f.write_fmt(format_args!("    {dep}\n"))?;
                }
            }
        }
        f.write_str("Symbols\n")?;
        for (symbol, paths) in self.symbols.iter() {
            symbol.fmt(f)?;
            fmt_delimited(&paths, " => ", ", ", "\n", f)?;
        }
        Ok(())
    }
}

impl<'a> Display for LabelView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (label, path) in self.0.labels.iter() {
            f.write_fmt(format_args!("## {label}\n"))?;
            if let Some(node) = self.0.node_at(path) {
                fmt_path_node("", path, node, f)?;
                for (path, node) in self.0.children(path) {
                    if node.labels.is_empty() {
                        fmt_path_node("", path, node, f)?;
                    } else {
                        f.write_fmt(format_args!("{path} "))?;
                        for label in node.labels.iter() {
                            f.write_fmt(format_args!(
                                "[&{label}](#{})",
                                label.to_string().replace("_", "")
                            ))?;
                        }
                        f.write_str("\n")?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Display for Xref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for sa in self.0.iter() {
            sa.fmt(f)?;
            "\n".fmt(f)?;
        }
        Ok(())
    }
}

impl Display for SymbolAssoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.first.fmt(f)?;
        if !self.others.is_empty() {
            fmt_delimited(&self.others, ", ", ", ", "", f)?;
        }
        "\n".fmt(f)?;
        fmt_path_node("    ", &self.path, &self.node, f)?;
        Ok(())
    }
}

pub fn fmt_path_node(
    indent: &str,
    path: &Path,
    node: &Node,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    indent.fmt(f)?;
    path.fmt(f)?;
    "\n".fmt(f)?;

    if !node.labels.is_empty() {
        indent.fmt(f)?;
        fmt_delimited(&node.labels, "  ", ", ", ":", f)?;
        "\n".fmt(f)?;
    }

    for prop in node.props.iter() {
        indent.fmt(f)?;
        "    ".fmt(f)?;
        prop.fmt(f)?;
        "\n".fmt(f)?;
    }
    Ok(())
}
