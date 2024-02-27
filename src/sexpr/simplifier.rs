use super::{Atom, Expr};

pub use super::Atom::*;

pub trait Simplifier {
    fn simplify(&self, subject: &Expr) -> Option<Expr>;
}

impl<F> Simplifier for F
where
    F: Fn(&Expr) -> Option<Expr>,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        (self)(subject)
    }
}

impl Simplifier for Atom {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        if subject.as_atom()? == self {
            Some(subject.clone())
        } else {
            None
        }
    }
}

impl Simplifier for &'static str {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        if subject.as_atom()?.as_symbol()? == *self {
            Some(subject.clone())
        } else {
            None
        }
    }
}

pub struct AnyNum;

impl Simplifier for AnyNum {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        if subject.as_atom()?.as_num().is_some() {
            Some(subject.clone())
        } else {
            None
        }
    }
}

pub struct AnyStr;

impl Simplifier for AnyStr {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        if subject.as_atom()?.as_string().is_some() {
            Some(subject.clone())
        } else {
            None
        }
    }
}

pub struct Anything;

impl Simplifier for Anything {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        Some(subject.clone())
    }
}

pub struct Nothing;

impl Simplifier for Nothing {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        if subject.is_empty() {
            Some(subject.clone())
        } else {
            None
        }
    }
}

pub struct Cons<A, B>(pub A, pub B);

impl<A, B> Simplifier for Cons<A, B>
where
    A: Simplifier,
    B: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        let elems = subject.as_list()?;
        let elem = elems.front()?;
        let head = self.0.simplify(elem)?;
        let mut elems = elems.clone();
        elems.pop_front();
        let tail = self.1.simplify(&Expr::List(elems))?;
        let l = if !head.is_empty() {
            let mut result = tail.into_deque()?;
            result.push_front(head);
            Expr::List(result)
        } else {
            tail
        };
        Some(l)
    }
}

pub struct Head<A>(pub A);

impl<A> Simplifier for Head<A>
where
    A: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        self.0.simplify(subject.as_list()?.front()?)
    }
}

pub struct Or<A, B>(pub A, pub B);

impl<A, B> Simplifier for Or<A, B>
where
    A: Simplifier,
    B: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        self.0
            .simplify(subject)
            .or_else(|| self.1.simplify(subject))
    }
}
pub struct And<A, B>(pub A, pub B);

impl<A, B> Simplifier for And<A, B>
where
    A: Simplifier,
    B: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        self.0.simplify(subject).and_then(|x| self.1.simplify(&x))
    }
}

pub struct Filter<A>(pub A);

impl<A> Simplifier for Filter<A>
where
    A: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        Some(Expr::list(
            subject.as_list()?.iter().filter_map(|x| self.0.simplify(x)),
        ))
    }
}

pub struct Find<A>(pub A);

impl<A> Simplifier for Find<A>
where
    A: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        subject.as_list()?.iter().find_map(|x| self.0.simplify(x))
    }
}

pub struct Discard<A>(pub A);

impl<A> Simplifier for Discard<A>
where
    A: Simplifier,
{
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        self.0.simplify(subject).is_some().then_some(Expr::empty())
    }
}

pub struct LabelAs(pub &'static str);

impl Simplifier for LabelAs {
    fn simplify(&self, subject: &Expr) -> Option<Expr> {
        Some(Expr::list([Expr::key(self.0), subject.clone()]))
    }
}
