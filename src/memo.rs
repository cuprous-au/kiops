use nom::{IResult, Parser};
use std::cell::OnceCell;

/// A lazy parser with erased type. Helps build recursive parsers.
#[derive(Default)]
pub struct Memo<'a, F, I, O, E> {
    build: F,
    parser: OnceCell<Box<dyn Parser<I, O, E> + 'a>>,
}

impl<'a, F, I, O, E> Memo<'a, F, I, O, E>
where
    Self: Parser<I, O, E>,
{
    pub fn new(f: F) -> Self {
        Self {
            build: f,
            parser: OnceCell::new(),
        }
    }
}

impl<'a, F, P, I, O, E> Parser<I, O, E> for Memo<'a, F, I, O, E>
where
    F: Fn() -> P,
    P: Parser<I, O, E> + 'a,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        // instaniate inner parser on first use and forget its exact type
        self.parser.get_or_init(|| Box::new((self.build)()));
        // use the inner parser via its dyn interface
        self.parser.get_mut().unwrap().parse(input)
    }
}
