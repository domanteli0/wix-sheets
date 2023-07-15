#![allow(incomplete_features)]

pub mod expr;
pub mod operators;
pub mod parse;
pub mod tests;

use derive_more::Display;
use serde_json::map::Map as SerdeMap;
use serde_json::value::Value as SerdeValue;
use std::convert::Into;
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

use self::expr::*;
use crate::{types::num::Num, types::value::Value};
use crate::data::{RawCellData, RawSheet};

/// Contains all cells of a sheet
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

    fn set_unchecked(&mut self, index: impl Into<Position>, expr: Expr) {
        let index = index.into();
        self.cells[index.y][index.x] = expr;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Display)]
#[display(fmt = "{}{}", "x + (b'A' as usize)", y)]
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
    #[error("#ERROR: Malformed formula")]
    ParseError,
    #[error("#ERROR: Incompatible types, expected {0}")]
    TypeMismatch(&'static str),
    #[error("#ERROR: Incompatible types")]
    BinaryTypeMismatch,
    #[error("#ERROR: This cell references an empty field")]
    InvalidReference,
    #[error("#ERROR: This operation takes {0:?} args, but {1} were supplied")]
    InvalidArgCount(std::ops::RangeInclusive<usize>, usize),
    #[error("#ERROR: Could not find an operation named {0}")]
    NoOpFound(String),
    #[error("#ERROR: Referenced cell {0} has errors {1:?}")]
    RefError(Box<CellError>, Position),
    #[error("#ERROR: These errors have occurred in this formula: {0:?}")]
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
    // instead of dropping the string then moving RawSheet in [Sheet::from]
    pub name: String,
    pub args: Vec<Expr>,
}

impl Sheet {
    // TODO: in case of reference cycles this implementation will cycle till the stack blows up, fix this
    /// Computes all fields, i.e. turns all values into constant values
    /// by computing formulas
    pub fn resolve_refs(&mut self, ops: &mut HashMap<&'static str, operators::Operator>) {
        for iy in 0..self.cells.len() {
            for jx in 0..self.cells[iy].len() {
                self.resolve_on_pos((jx, iy).into(), ops);
            }
        }
    }

    fn resolve_on_pos(
        &mut self,
        pos: Position,
        ops: &mut HashMap<&'static str, operators::Operator>,
    ) -> Option<&Expr> {
        let expr = self.get(pos)?;

        let new_expr: Expr = match expr.clone() {
            Expr::Ref(r) => self
                .resolve_on_pos(r, ops)
                .map(|e| e)
                .unwrap_or(&Expr::Err(CellError::InvalidReference))
                .clone(),
            Expr::Form(mut op_info) => {
                op_info.resolve_with_sheet(self, ops);

                ops.get_mut(&op_info.name[..])
                    .map(|o| o(self, &mut op_info))
                    .unwrap_or(CellError::NoOpFound(op_info.name.clone()).into())
            }
            Expr::Value(v) => v.into(),
            Expr::Err(e) => Expr::Err(CellError::RefError(Box::new(e), Position { x: 0, y: 0 })),
        };

        self.set_unchecked(pos, new_expr);
        self.get(pos)
    }
}

impl OpInfo {
    // after this is called `self` should only contain
    // `Expr`s which are either `Err` or `Value`
    fn resolve_with_sheet(
        &mut self,
        sheet: &mut Sheet,
        ops: &mut HashMap<&'static str, operators::Operator>,
    ) {
        let mut self_ = self.clone();
        self_.args.iter_mut().for_each(|e: &mut Expr| {
            if e.is_err() || e.is_value() {
                return;
            }

            let e_ = match e {
                Expr::Ref(r) => sheet
                    .resolve_on_pos(*r, ops)
                    .map(|e| e)
                    .unwrap_or(&Expr::Err(CellError::InvalidReference))
                    .clone(),
                Expr::Form(op_info) => {
                    op_info.resolve_with_sheet(sheet, ops);

                    ops.get_mut(&op_info.name[..])
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
    fn from(value: RawSheet) -> Self {
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
            RawCellData::Int(i) => Expr::Value(Num::I(i).into()),
            RawCellData::Float(f) => Expr::Value(Num::F(f).into()),
            RawCellData::Bool(b) => Expr::Value(b.into()),
            RawCellData::String(s) => match parse::parse_entry(&s[..]) {
                Ok((_, expr)) => expr,
                Err(_) => Expr::Err(CellError::ParseError),
            },
        }
    }
}

/// This impl is used for serialization
impl Into<SerdeValue> for Sheet {
    fn into(self) -> SerdeValue {
        let data = self
            .cells
            .iter()
            .map(|row| {
                SerdeValue::Array(
                    row.iter()
                        .map(|cell| cell.clone().into())
                        .collect::<Vec<SerdeValue>>(),
                )
            })
            .collect::<Vec<_>>();

        SerdeValue::Object(SerdeMap::from_iter([
            ("id".to_owned(), SerdeValue::String(self.id.clone())),
            ("data".to_owned(), SerdeValue::Array(data)),
        ]))
    }
}
