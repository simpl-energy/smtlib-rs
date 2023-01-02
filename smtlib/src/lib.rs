#![cfg_attr(feature = "const-bit-vec", feature(generic_const_exprs))]

//! # smtlib
//!
//! _A high-level API for interacting with SMT solvers._

use std::collections::{hash_map::Entry, HashMap};

use itertools::Itertools;
use smtlib_lowlevel::{
    ast::{self, Identifier, QualIdentifier},
    lexicon::Symbol,
    Driver,
};
use terms::{Const, Sort};

pub use logics::Logic;
pub use smtlib_lowlevel::{backend, Backend};

mod logics;
pub mod terms;
pub mod theories;

pub use theories::{core::*, fixed_size_bit_vectors::*, ints::*, reals::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SatResult {
    Unsat,
    Sat,
    Unknown,
}

#[cfg(test)]
mod tests {
    use crate::terms::{forall, Sort};

    use super::*;

    #[test]
    fn int_math() {
        let x = Int::from_name("x");
        let y = Int::from_name("hello");
        // let x_named = x.labeled();
        let mut z = 12 + y * 4;
        z += 3;
        let w = x * x + z;
        println!("{w}");
    }

    #[test]
    fn quantifiers() {
        let x = Int::from_name("x");
        let y = Int::from_name("y");

        let res = forall((x, y), (x + 2)._eq(y));
        println!("{}", ast::Term::from(res));
    }
}

#[derive(Debug)]
pub struct Solver<B> {
    driver: Driver<B>,
    decls: HashMap<Identifier, ast::Sort>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lowlevel(
        #[from]
        #[diagnostic_source]
        smtlib_lowlevel::Error,
    ),
    #[error("SMT error {0} after running {1}")]
    Smt(String, String),
}

impl<B> Solver<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Result<Self, Error> {
        Ok(Self {
            driver: Driver::new(backend)?,
            decls: Default::default(),
        })
    }
    pub fn set_logic(&mut self, logic: Logic) -> Result<(), Error> {
        let cmd = ast::Command::SetLogic(Symbol(logic.to_string()));
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::Success => Ok(()),
            ast::GeneralResponse::SpecificSuccessResponse(_) => todo!(),
            ast::GeneralResponse::Unsupported => todo!(),
            ast::GeneralResponse::Error(_) => todo!(),
        }
    }
    pub fn assert(&mut self, b: Bool) -> Result<(), Error> {
        let term = ast::Term::from(b);
        for q in term.all_consts() {
            match q {
                QualIdentifier::Identifier(_) => {}
                QualIdentifier::Sorted(i, s) => match self.decls.entry(i.clone()) {
                    Entry::Occupied(stored) => assert_eq!(s, stored.get()),
                    Entry::Vacant(v) => {
                        v.insert(s.clone());
                        match i {
                            Identifier::Simple(sym) => {
                                self.driver
                                    .exec(&ast::Command::DeclareConst(sym.clone(), s.clone()))?;
                            }
                            Identifier::Indexed(_, _) => todo!(),
                        }
                    }
                },
            }
        }
        let cmd = ast::Command::Assert(term);
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::Success => Ok(()),
            ast::GeneralResponse::Error(e) => Err(Error::Smt(e, cmd.to_string())),
            _ => todo!(),
        }
    }
    pub fn check_sat(&mut self) -> Result<SatResult, Error> {
        let cmd = ast::Command::CheckSat;
        match self.driver.exec(&cmd)? {
            ast::GeneralResponse::SpecificSuccessResponse(
                ast::SpecificSuccessResponse::CheckSatResponse(res),
            ) => Ok(match res {
                ast::CheckSatResponse::Sat => SatResult::Sat,
                ast::CheckSatResponse::Unsat => SatResult::Unsat,
                ast::CheckSatResponse::Unknown => SatResult::Unknown,
            }),
            ast::GeneralResponse::Error(msg) => Err(Error::Smt(msg, format!("{cmd}"))),
            res => todo!("{res:?}"),
        }
    }
    pub fn get_model(&mut self) -> Result<Model, Error> {
        match self.driver.exec(&ast::Command::GetModel)? {
            ast::GeneralResponse::SpecificSuccessResponse(
                ast::SpecificSuccessResponse::GetModelResponse(model),
            ) => Ok(Model::new(model)),
            res => todo!("{res:?}"),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Model {
    values: HashMap<String, ast::Term>,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.values.fmt(f)
    }
}
impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {} }}",
            self.values
                .iter()
                .map(|(n, t)| format!("{n}: {t}"))
                .format(", ")
        )
    }
}

impl Model {
    fn new(model: ast::GetModelResponse) -> Self {
        Self {
            values: model
                .0
                .into_iter()
                .map(|res| match res {
                    ast::ModelResponse::DefineFun(f) => (f.0 .0.trim_matches('|').into(), f.3),
                    ast::ModelResponse::DefineFunRec(_) => todo!(),
                    ast::ModelResponse::DefineFunsRec(_, _) => todo!(),
                })
                .collect(),
        }
    }
    pub fn eval<T: Sort + std::fmt::Debug>(&self, x: Const<T>) -> Option<T::Inner>
    where
        T::Inner: From<ast::Term>,
    {
        Some(self.values.get(x.name().trim_matches('|'))?.clone().into())
    }
}