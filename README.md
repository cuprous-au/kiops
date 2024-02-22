# Extracting data from KiCAD files

This repo contains experimental software for parsing [KiCAD](https://kicad.org/) electronic design files and extracting information from them.  There is also a parser for [device tree](https://devicetree.org) files and some miscellaneous _nushell_ helper scripts.

KiCAD schematic, PCB, footprint, and symbol files are in [S-expression](https://en.wikipedia.org/wiki/S-expression) form.  The parser here was adapted from the example in the [Nom](https://github.com/rust-bakery/nom) project. 

The structure of a device tree is similar but the syntax is different. There is a separate parser for those.

Once parsed, the KiCAD data can be transformed and then serialized as a new S-expression.  The parser/serializer are good enough to roundtrip the projects I have on hand without loss.

```
    KiCAD file -> parse -> transform -> serialize -> output file
```

The transform step could be used to make a systematic change that preserves the structure, in which case the output file would be a valid KiCAD file.  Or it could be extracting information, such as symbol or footprint names, in which case the output would be a summary.  A limitation, for now, is that the output will always be an S-expression.

## Data Structure

The core data structure looks like this:

```rust
pub enum Atom {
    Symbol(String),
    Str(String),
    Num(f64),
    Bits(u64),
    Bool(bool),
    Uuid(Uuid),
}

pub enum Expr {
    Constant(Atom),
    List(VecDeque<Expr>),
}
```

The `From<X>` trait is implemented for `Atom` and `Expr` for various types `X` and extraction methods yielding `Option<X>` are provided.

## Simplifiers

The `Simplifier` trait makes it easy to extract information from an `Expr` in many cases.  

```rust
pub trait Simplifier {
    fn simplify(&self, subject: &Expr) -> Option<Expr>;
}
```

Any function `Fn(&Expr) -> Option<Expr>` is a simplifier (a blanket implementation is provided).  For example, to match a `Expr::Constant(Atom::Str(_))` and trim white space:

```rust
pub fn trim(x: &Expr) -> Option<Expr> {
    Some(x.is_atom()?.string()?.trim().into())
}
```

Simplifiers can be combined.  For example, KiCAD symbols and footprints have _properties_.  Here is a simplifier that matches any property and trims its value:

```rust
let property = Cons("property", Cons(AnyStr, Cons(trim, Anything)))
```

This uses a combinator called `Cons` that applies one simplifier to the head and another to the tail of an `Expr::List(_)`.

Some of the information in a KiCAD property concerns how and where to display it. This comes after the property value and is matched by `Anything` in the above.  If we want to discard that information:

```rust
let property_kv_only = Cons("property", Cons(AnyStr, Cons(trim, Discard(Anything))))
```

Putting it all together, this example consumes a complete schematic diagram and reduces it to symbols and their properties.

```rust
pub fn extract_symbols(schematic: &Expr) -> Option<Expr> {
    let property_kv_only = Cons("property", Cons(AnyStr, Cons(trim, Discard(Anything))));

    let symbols = Cons(
        "kicad_sch",
        Filter(Cons("symbol", Filter(property_kv_only))),
    );
    
    symbols.simplify(schematic)
}
```

## Table of Simplifiers and Combinators

|Simplifier|Description|
|---|---|
|`AnyNum`| match a `Constant(Num(_))`|
|`AntStr`| match a `Constant(Str(_))`|
|`a: Atom`| match a `Constant(a)`|
|`s: &'static str`| match a `Constant(Symbol(s))`|
|`Anything`| match any expression|
|`Nothing`| match an empty `List`|

|Combinator|Description|
|---|---|
|`Cons(H,T)`| match the head element of a `List` with `H` and the tail with `T` producing a new `List`|
|`Filter(X)`| match each element of a `List` with `X` producing a `List` of the successes|
|`Or(X,Y)`| match with `X` or, if that fails, match with `Y`|
|`And(X,Y)`| match with `X` and, if that succeeds, match the simplified result with `Y`|
|`Discard(X)`| match with `X` and, if that succeeds, replace with an empty `List`|

Special case: `Cons(Discard(H), T)` produces the result of `T` if `H` and `T` succeed.

### Footnote about rust pattern matching

The motivation for the `Simplifier` trait is the difficulty of pattern matching over a recursive  data structure in rust.  These will always contain smart pointers such as `Rc`, `Box` or `Vec` which cannot be destructured in a `match`. 

The work around, short of writing a lot of boilerplate, is to provide helpers such as `Expr::is_atom()` and `Simplifier::simplify(_)` that return `Option`.   Pattern matching with recursion is bread and butter functional programming style and one day it will be available in rust. But not yet.
