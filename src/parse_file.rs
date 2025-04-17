use nom::error::VerboseError;
use nom::{InputTake, Parser};
use serde_json::Value;
use std::fmt::Display;
use std::fs::File;
use std::io::{stdin, stdout, BufReader, Read, Write};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_file<P, T>(name: &str, parser: P) -> Result<T>
where
    P: for<'a> Parser<&'a str, T, VerboseError<&'a str>>,
    T: 'static,
{
    let mut f = File::open(name)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    parse_with(&buf, parser)
}

pub fn write_file<A: Display>(path: &str, content: &A) -> Result<()> {
    let mut f = File::create(path)?;
    f.write_all(content.to_string().as_bytes())?;
    Ok(())
}

pub fn parse_stdin<P, T>(parser: P) -> Result<T>
where
    P: for<'a> Parser<&'a str, T, VerboseError<&'a str>>,
    T: 'static,
{
    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    parse_with(&buf, parser)
}

pub fn write_stdout<A: Display>(content: &A) -> Result<()> {
    stdout().write_all(content.to_string().as_bytes())?;
    Ok(())
}

/// Read a serde_json `Value` from a file
pub fn read_json(path: &str) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file.
    let value = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(value)
}

pub fn parse_with<P, T>(text: &str, mut parser: P) -> Result<T>
where
    P: for<'a> Parser<&'a str, T, VerboseError<&'a str>>,
    T: 'static,
{
    Ok(parser
        .parse(text)
        .map(|(_, data)| data)
        .map_err(|err| err.map(abrev))?)
}

fn abrev(err: VerboseError<&str>) -> VerboseError<String> {
    let errors = err
        .errors
        .into_iter()
        .map(|(i, k)| (i.take(160.min(i.len())).to_owned(), k))
        .collect();
    VerboseError { errors }
}
