use std::fmt::Display;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};

use nom::error::VerboseError;
use nom::{IResult, Parser};

pub fn parse_file<P, T>(name: &str, mut parser: P) -> Result<T, ()>
where
    P: for<'a> Parser<&'a str, T, VerboseError<&'a str>>,
    T: 'static,
{
    let mut f = File::open(name).expect("could not open file");
    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("could not read file");
    extract_output(parser.parse(&buf))
}

pub fn write_file<A: Display>(path: &str, content: &A) {
    let mut f = File::create(path).expect("could not create file");
    f.write_all(content.to_string().as_bytes())
        .expect("could not write file");
}

pub fn write_stdout<A: Display>(content: &A) {
    stdout()
        .write_all(content.to_string().as_bytes())
        .expect("could not write to output");
}

pub fn parse_stdin<P, T>(mut parser: P) -> Result<T, ()>
where
    P: for<'a> Parser<&'a str, T, VerboseError<&'a str>>,
    T: 'static,
{
    let mut buf = String::new();
    stdin()
        .read_to_string(&mut buf)
        .expect("could not read stdin");
    extract_output(parser.parse(&buf))
}

pub fn extract_output<T>(result: IResult<&str, T, VerboseError<&str>>) -> Result<T, ()> {
    result.map(|(_, t)| t).map_err(print_nom_err)
}

pub fn print_nom_err(err: nom::Err<VerboseError<&str>>) {
    match err {
        nom::Err::Incomplete(_) => println!("incomplete"),
        nom::Err::Error(err) => {
            println!("{}", abrev(err))
        }
        nom::Err::Failure(err) => {
            println!("{}", abrev(err))
        }
    }
}

fn abrev(err: VerboseError<&str>) -> VerboseError<&str> {
    let errors = err
        .errors
        .into_iter()
        .map(|(i, k)| (&i[0..160], k))
        .collect();
    VerboseError { errors }
}
