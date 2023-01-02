#![doc = concat!("```ignore\n", include_str!("./Reals.smt2"), "```")]

use smtlib_lowlevel::{
    ast::{self, Identifier, Term},
    lexicon::Symbol,
};

use crate::{
    impl_op,
    terms::{fun, qual_ident, Const, Dynamic, Sort},
    Bool,
};

#[derive(Debug, Clone, Copy)]
pub struct Real(&'static Term);
impl From<Const<Real>> for Real {
    fn from(c: Const<Real>) -> Self {
        c.1
    }
}
impl std::fmt::Display for Real {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Term::from(*self).fmt(f)
    }
}

impl From<Real> for Dynamic {
    fn from(i: Real) -> Self {
        Term::from(i).into()
    }
}

impl From<Real> for Term {
    fn from(i: Real) -> Self {
        i.0.clone()
    }
}
impl From<Term> for Real {
    fn from(t: Term) -> Self {
        Real(Box::leak(Box::new(t)))
    }
}
impl Sort for Real {
    type Inner = Self;
    fn sort() -> ast::Sort {
        ast::Sort::Sort(Identifier::Simple(Symbol("Real".into())))
    }
}
impl From<i64> for Real {
    fn from(i: i64) -> Self {
        Term::Identifier(qual_ident(i.to_string(), None)).into()
    }
}
impl From<f64> for Real {
    fn from(i: f64) -> Self {
        Term::Identifier(qual_ident(i.to_string(), None)).into()
    }
}
impl Real {
    fn binop<T: From<Term>>(self, op: &str, other: Real) -> T {
        fun(op, vec![self.into(), other.into()]).into()
    }
    pub fn gt(self, other: impl Into<Self>) -> Bool {
        self.binop(">", other.into())
    }
    pub fn ge(self, other: impl Into<Self>) -> Bool {
        self.binop(">=", other.into())
    }
    pub fn lt(self, other: impl Into<Self>) -> Bool {
        self.binop("<", other.into())
    }
    pub fn le(self, other: impl Into<Self>) -> Bool {
        self.binop("<=", other.into())
    }
    pub fn abs(self) -> Real {
        fun("abs", vec![self.into()]).into()
    }
}

impl std::ops::Neg for Real {
    type Output = Self;
    fn neg(self) -> Self::Output {
        fun("-", vec![self.into()]).into()
    }
}

impl_op!(Real, f64, Add, add, "+", AddAssign, add_assign, +);
impl_op!(Real, f64, Sub, sub, "-", SubAssign, sub_assign, -);
impl_op!(Real, f64, Mul, mul, "*", MulAssign, mul_assign, *);
impl_op!(Real, f64, Div, div, "div", DivAssign, div_assign, /);