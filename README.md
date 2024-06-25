# Extracting data from KiCAD files

This repo contains experimental software for parsing [KiCAD](https://kicad.org/) electronic design files and extracting information from them.  There is also a parser for [device tree](https://devicetree.org) files and some _nushell_ helper scripts.

If you need a quick start to generate BOM and other fabrication files see: `BUILD.md`

KiCAD schematic, PCB, footprint, and symbol files are in [S-expression](https://en.wikipedia.org/wiki/S-expression) form.  The parser here was adapted from the example in the [Nom](https://github.com/rust-bakery/nom) project. 

The structure of a device tree is similar but the syntax is different. There is a separate parser for those.

Once parsed, the KiCAD data can be transformed and then serialized.  The parser/serializer are good enough to roundtrip the projects I have on hand without loss.

```
    KiCAD file -> parse -> transform -> serialize -> output file
```

The transform step could be used to make a systematic change that preserves the structure, in which case the output could be a valid KiCAD file.  Or it could be extracting information, such as symbol or footprint names, in which case the output would be a summary.  

The output format can be an S-expression or JSON.  A JSON conversion is provided that does a reasonable job of finding objects in among the nested lists of an S-expression.

## Command line Usage

A basic executable is provided to summarize footprints and symbols and to convert KiCAD files to JSON.  But the software is more flexible when used as a library.  

Commands:
```sh
./ki_parse footprints
./ki_parse symbols
./ki_parse sheets
./ki_parse format
./ki_merge symbol-library
./dts_parse
```

### `ki_parse`

 - `footprints` takes a PCB (`.kicad_pcb`) file on the standard input and produces a JSON summary of the footprints found.  
 - `symbols` takes a schematic (`.kicad_sch`) file and summarizes its symbols 
 - `sheets` summarizes schematic sheets 
 - `format` command takes either type of file and produces a JSON translation 

Appending `-s` to these commands produces an S-expression instead of JSON.

### `ki_merge`

This takes a symbol library on its standard input and the file name of another symbol library as its argument. Their contents are merged producing a new symbol library on the standard output.   

The two libraries must have the same _version_ and _generator_ attributes.  (Use the KiCAD CLI to upgrade libraries.) Duplicate symbols are eliminated, the first symbol with a given name is kept.

### `dts_parse` 

This command takes device tree source and produces a JSON rendition of it. 

## `nushell` module

A [`nushell`](nushell.sh) module provides higher level commands.  

```nushell
nushell> use kiops.nu
nushell> $env.kiops_lib_location = <kicad-libs-location>
nushell> kiops upgrade footprints
nushell> kiops upgrade symlibs
nushell> kiops merge symlibs
nushell> kiops install libs <project_dir>
nushell> kiops survey symbols <project_dir>
nushell> kiops survey footprints <project_dir>
```

## Data Structure

The core data structure for KiCAD S-expressions looks like this:

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

The `From<X>` trait is implemented for `Atom` and `Expr` with various types `X` and extraction methods `as_X` yielding `Option<X>` are defined.

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
    Some(x.as_atom()?.as_string()?.trim().into())
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
|`Head(H)`|  match the head element of a `List` with `H` extracting this from the list|
|`Filter(X)`| match each element of a `List` with `X` producing a `List` of the successes|
|`Find(X)` | match each element of a `List` with `X` extracting the first success from the list |
|`Or(X,Y)`| match with `X` or, if that fails, match with `Y`|
|`And(X,Y)`| match with `X` and, if that succeeds, match the simplified result with `Y`|
|`Discard(X)`| match with `X` and, if that succeeds, replace with an empty `List`|

Special case: `Cons(Discard(H), T)` produces the result of `T` if `H` and `T` succeed.

## Rationale for `Simplifier`

The motivation for the `Simplifier` trait is that pattern matching over a recursive  data structure in is inconvenient in rust.  These will always contain smart pointers such as `Rc`, `Box` or `Vec` which cannot be destructured in a `match`. 

We can write nested `match` expressions but helpers such as `Expr::as_list()`, `Expr::as_atom()` that return `Option` work better. Rust likes `Option`s.  The ultimate such helper is `Simplifier::simplify(_)` which enables the combinator scheme here. 

Pattern matching over recursive structures is bread and butter functional programming style and one day it will be available in rust. But not yet.
