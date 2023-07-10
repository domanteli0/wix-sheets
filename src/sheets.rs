#![allow(incomplete_features)]

pub mod num;
pub mod parse;
pub mod tests;
pub mod value;
pub mod formulas;

use serde::de::IntoDeserializer;
use std::{collections::HashMap, fmt::Debug};
use std::ops::{RangeBounds, Bound};
use thiserror::Error;
// use serde::{Serialize, Serializer};
use derive_more::{From, IsVariant, Unwrap};

use self::{num::Num, value::Value};
use crate::data::{RawCellData, RawSheet};

#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub id: String,
    pub cells: Vec<Vec<Expr>>,
}

impl Sheet {
    fn get(&self, index: impl Into<Position>) -> Option<&Expr> {
        let index = index.into();
        self.cells
            .get(index.y)
            .and_then(|row: &Vec<_>| row.get(index.x))
    }

    fn get_mut(&mut self, index: impl Into<Position>) -> Option<&mut Expr> {
        let index = index.into();
        self.cells
            .get_mut(index.y)
            .and_then(|row: &mut Vec<_>| row.get_mut(index.x))
    }

    fn set_unchecked(&mut self, index: impl Into<Position>, expr: Expr) {
        let index = index.into();
        self.cells[index.y][index.x] = expr;
    }
}

#[derive(Debug, Clone, From, PartialEq, IsVariant, Unwrap)]
pub enum Expr {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo),
    Err(CellError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CellError {
    #[error("$ERROR: Malformed formula")]
    ParseError,
    #[error("#ERROR: Incompatible types, expected {0}")]
    TypeMismatch(&'static str),
    #[error("#ERROR: This cell references an empty field")]
    InvalidReference,
    #[error("#ERROR: This operation takes {0:?} args, but {1} were supplied")]
    // TODO: Select a concrete Range impl and use that
    InvalidArgCount(core::ops::Bound<usize>, usize),
    #[error("#ERROR: Could not find an operation named {0}")]
    NoOpFound(String),
    #[error("$ERROR: Referenced cell {0} has errors {1:?}")]
    RefError(Box<CellError>, Position),
    #[error("$#ERROR: These errors have occured in this formula: {0:?}")]
    // usize - which arg, CellError - what type of error
    FormError(Vec<(usize, CellError)>),
    #[error("#ERROR: Division by zero")]
    DivByZero,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpInfo {
    // This could be a `&str` but then `RawCellData` needs to
    // to take a reference to `str` too instead holding a `String`
    // in order to take over the reference
    // instead of droping the string then moving RawSheet in [Sheet::from]
    pub name: String,
    pub args: Vec<Expr>,
}

pub type Form = Box<dyn Fn(&mut Sheet, &mut OpInfo) -> Expr>;

fn find_errs<C: Value + Clone>(self_: &mut OpInfo, type_err: &'static str) -> Vec<(usize, CellError)> {
    self_
        .args
        .iter()
        .enumerate()
        .filter(|(_, expr)|
            expr.is_err() || expr.clone().clone().unwrap_value().downcast_ref::<C>().is_none()
        )
        .map(|(ix, expr): (_, &Expr)| match expr {
            Expr::Value(v) => (ix + 1, CellError::TypeMismatch(type_err)),
            Expr::Err(e) => (ix + 1, e.clone()),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>() 
}

fn foldr<C: Value + Clone, T>(
    self_: &mut OpInfo,
    init: T,
    f: impl Fn(T, C) -> T,
    type_err: &'static str,
) -> Result<T, CellError> {
    // find and return errors
    let errors = find_errs::<C>(self_, type_err);

    if errors.len() > 0 {
        return Err(CellError::FormError(errors));
    }

    Ok(self_.args.iter().fold(init, |acc, e| {
        f(acc, e.clone().unwrap_value().downcast_ref::<C>().unwrap().clone())
    }))
}

fn foldr_with_check<C: Value + Clone, T: Value>(
    self_: &mut OpInfo,
    init: T,
    f: impl Fn(T, C) -> T,
    type_err: &'static str,
    acceptable_range: impl RangeBounds<usize>,
) -> Expr {
    let to_owned_bound = |b: Bound<&usize>| -> Bound<usize> {
        match b {
            Bound::Included(i) => Bound::Included(*i),
            Bound::Excluded(i) => Bound::Excluded(*i),
            Bound::Unbounded => Bound::Unbounded, 
        }
    };

    if !acceptable_range.contains(&self_.args.len()) {
        return Expr::Err(
            CellError::InvalidArgCount(to_owned_bound(acceptable_range.start_bound()), self_.args.len())
        );
    }

    foldr(
        self_, 
        init,
        f,
        type_err,
    )
        .map(|n| Expr::Value(Box::new(n)))
        .unwrap_or_else(|e| Expr::Err(e)) 
}

impl Sheet {
    fn get_form_map<'a>() -> HashMap<&'a str, Form> {
        let mut map = HashMap::<&str, Form>::new();

        let sum = Box::new(
            |sheet: &mut Sheet, info: &mut OpInfo| {
                foldr_with_check(
                    info,
                    Num::I(0),
                    |acc, n| acc + n,
                    "Num",
                    1..
                )
            }
        );

        let mul: Form = Box::new(|sheet, info| {
            foldr_with_check(
                info,
                Num::I(1),
                |acc, n| acc * n,
                "Num",
                1..)
        });

        let div: Form = Box::new(|sheet, info| {
            let errs = find_errs::<Num>(info, "Num");
            if errs.len() > 0 { return Expr::Err(CellError::FormError(errs)); }

            if info.args.len() != 2 { 
                return  Expr::Err(
                    CellError::InvalidArgCount(Bound::Included(2),
                    info.args.len())
                );
            }

            let arg1: Num = *info.args[0].clone().unwrap_value().downcast_ref::<Num>().unwrap();
            let arg2 = *info.args[1].clone().unwrap_value().downcast_ref::<Num>().unwrap();
            

            if arg2 == Num::I(0) || arg2 == Num::F(0.0) { return Expr::Err(CellError::DivByZero)}

            Expr::Value(Box::new(
                arg1 / arg2
            ))
        });

        map.insert("SUM", sum);
        map.insert("MULTIPLY", mul);
        map.insert("DIVIDE", div);

        map
    }

    // TODO: in case of reference cycles this implementation will cycle till the stack blows up, fix this
    pub fn resolve_refs(&mut self) {
        for iy in 0..self.cells.len() {
            for jx in 0..self.cells[iy].len() {
                self.resolve_on_pos(dbg!((jx, iy).into()));
            }
        }
    }

    fn resolve_on_pos(&mut self, pos: Position) -> Option<&Expr> {
        let expr = self.get(pos)?;

        let new_expr: Expr = match expr.clone() {
            Expr::Ref(r) => self
                .resolve_on_pos(r)
                .map(|e| e)
                .unwrap_or(&Expr::Err(CellError::InvalidReference))
                .clone(),
            Expr::Form(mut op_info) => {
                let mut map = Self::get_form_map();

                op_info.resolve_with_sheet(self);

                map.get_mut(&op_info.name[..])
                    .map(|o| o(self, &mut op_info))
                    .unwrap_or(CellError::NoOpFound(op_info.name.clone()).into())
            }
            Expr::Value(v) => v.into(),
            // TODO: actual position
            Expr::Err(e) => Expr::Err(CellError::RefError(Box::new(e), Position { x: 0, y: 0 })),
        };

        self.set_unchecked(pos, new_expr);
        self.get(pos)
    }
}

impl OpInfo {
    // after this is called `self` should only contain
    // `Expr`s which are either `Err` or `Value`
    fn resolve_with_sheet(&mut self, sheet: &mut Sheet) {
        let mut self_ = self.clone();
        self_.args.iter_mut().for_each(|e: &mut Expr| {
            if e.is_err() || e.is_value() {
                return;
            }

            let e_ = match e {
                Expr::Ref(r) => sheet
                    .resolve_on_pos(*r)
                    .map(|e| e)
                    .unwrap_or(&Expr::Err(CellError::InvalidReference))
                    .clone(),
                Expr::Form(op_info) => {
                    op_info.resolve_with_sheet(sheet);

                    // actually calc
                    // TODO: move this somewhere else as not to re-create
                    // the whole hashmap everytime
                    Sheet::get_form_map()
                        .get_mut(&op_info.name[..])
                        .map(|o| o(sheet, op_info))
                        .unwrap_or(CellError::NoOpFound(op_info.name.clone()).into())
                }
                _ => unreachable!(),
            };

            *e = e_;
        });

        *self = self_;
    }
}

impl<'a> From<RawSheet> for Sheet {
    fn from(mut value: RawSheet) -> Self {
        let cells = value
            .data
            .into_iter()
            .map(|raw_row| raw_row.into_iter().map(RawCellData::into).collect())
            .collect();

        Self {
            id: value.id,
            cells,
        }
    }
}

impl<'a> From<RawCellData> for Expr {
    fn from(value: RawCellData) -> Self {
        match value {
            RawCellData::Int(i) => Expr::Value(Box::new(Num::I(i))),
            RawCellData::Float(f) => Expr::Value(Box::new(Num::F(f))),
            RawCellData::Bool(b) => Expr::Value(Box::new(b)),
            RawCellData::String(s) => match parse::parse_entry(&s[..]) {
                Ok((_, expr)) => expr,
                Err(_) => Expr::Err(CellError::ParseError),
            },
        }
    }
}
