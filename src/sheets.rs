#![allow(incomplete_features)]

pub mod num;
pub mod parse;
pub mod tests;
pub mod value;

use std::{collections::HashMap, fmt::Debug};
use serde::de::IntoDeserializer;
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
    #[error("#ERROR: Incompatible types, expected {0} at {1}")]
    TypeMismatch(&'static str, usize),
    #[error("#ERROR: This cell references an empty field")]
    InvalidReference,
    #[error("#ERROR: This operation takes {0} args, but {1} were supplied")]
    InvalidArgCount(usize, usize),
    #[error("#ERROR: Could not find an operation named {0}")]
    NoOpFound(String),
    #[error("$ERROR: Referenced cell(s) have errors {0:?}")]
    RefError(Vec<(Position, CellError)>)
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

fn get_at_least_of_type<T: Value>(Vec)

impl Sheet {
    fn get_form_map<'a>() -> HashMap<&'a str, Form> {
        let mut map = HashMap::<&str, Form>::new();
        // let get_args = |op_info: &OpInfo| -> Result<(), CellError> {

        //     todo!();
        // };

        // let expect_type = |op_info: &OpInfo| -> Result<(), CellError> {

        //     todo!()
        // };

        // Add one or more values together.
        let sum: Form = Box::new(|sheet, info| {
            if info.args.len() < 1 {
                return CellError::InvalidArgCount(1, 0).into();
            }

            let val_to_num_or_err =
                |val: &Box<dyn Value>, num: Num, pos: usize| -> Result<Num, CellError> {
                match val.clone().downcast_ref::<Num>() {
                    Some(n) => Ok((*n + num).into()),
                    None => Err(CellError::TypeMismatch("Num", pos + 1)),
                }
            };

            let mut add = |num: Num, expr: &mut Expr, pos: usize| -> Result<Num, CellError> {
                match expr {
                    Expr::Value(v) => {
                        val_to_num_or_err(v, num, pos)
                    },
                    Expr::Err(e) => 
                        // TODO: Actually use ref position for `RefError` 
                        Err(CellError::RefError(vec![((0,0).into(), e.clone())])),
                    _ => unreachable!()
                }
            };

            let mut res_fn = || -> Result<_, _> {
                let mut res = Ok(Num::I(0));
                for (pos, expr) in info.args.iter_mut().enumerate() {
                    let res_ = res?;
                    res = add(res_, expr, pos);
                }
                res
            };

            match res_fn() {
                Ok(num) => num.into(),
                Err(r) => r.into(),
            }
        });

        map.insert("SUM", sum);

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
                    .unwrap_or(
                        CellError::NoOpFound(
                            op_info.name.clone()
                        ).into()
                    )
            },
            Expr::Value(v) => v.into(),
            Expr::Err(e) => Expr::Err(
                CellError::RefError(vec![((0,0).into(), e)])
            ),
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
            if e.is_err() || e.is_value() { return; }

            let e_ = match e {
                Expr::Ref(r) => sheet
                    .resolve_on_pos(*r)
                    .map(|e| e)
                    .unwrap_or(
                        &Expr::Err(CellError::InvalidReference)
                    )
                    .clone(),
                Expr::Form(op_info) => {
                    op_info.resolve_with_sheet(sheet);

                    // actually calc
                    // TODO: move this somewhere else as not to re-create
                    // the whole hashmap everytime
                    Sheet::get_form_map()
                        .get_mut(&op_info.name[..])
                        .map(|o| o(sheet, op_info))
                        .unwrap_or(
                            CellError::NoOpFound(
                                op_info.name.clone()
                            ).into()
                        )
                },
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
