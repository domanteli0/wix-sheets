#![allow(incomplete_features)]

pub mod num;
pub mod operators;
pub mod parse;
pub mod tests;
pub mod value;

use derive_more::{From, IsVariant, Unwrap};
use dyn_ord::{DynEq, DynOrd};
use jsonway::{self, ObjectBuilder};
use serde_json::value::Value as SerdeValue;
use serde_json::map::Map as SerdeMap;
use serde_json::Number;
use std::convert::Into;
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

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

    fn set_unchecked(&mut self, index: impl Into<Position>, expr: Expr) {
        let index = index.into();
        self.cells[index.y][index.x] = expr;
    }
}

#[derive(Debug, Clone, From, IsVariant, Unwrap)]
// TODO: implement unwrap_value such that it moves self, reducing cloning
pub enum Expr {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo),
    Err(CellError),
}

impl PartialEq for Expr {
    fn eq(&self, rhs: &Expr) -> bool {
        if self.is_err() && rhs.is_err() {
            self.clone().unwrap_err() == rhs.clone().unwrap_err()
        } else if self.clone().is_form() && rhs.clone().is_form() {
            self.clone().unwrap_form() == rhs.clone().unwrap_form()
        } else if self.is_ref() && rhs.is_ref() {
            self.clone().unwrap_ref() == rhs.clone().unwrap_ref()
        } else if self.is_value() == rhs.is_value() {
            (self.clone().unwrap_value() as Box<dyn DynEq>)
                == (rhs.clone().unwrap_value() as Box<dyn DynEq>)
        } else {
            false
        }
    }
}

impl Into<SerdeValue> for Expr {
    fn into(self) -> SerdeValue {
        match self {
            Expr::Value(v) => {
                if let Some(b) = v.downcast_ref::<bool>() {
                    return SerdeValue::Bool(*b);
                }
                if let Some(n) = v.downcast_ref::<Num>() {
                    return SerdeValue::Number(Number::from_f64((*n).into()).unwrap());
                }
                SerdeValue::String(v.to_string())
            }
            Expr::Err(e) => serde_json::value::Value::String(e.to_string()),
            _ => unreachable!("Assumed resolved"),
        }
    }
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
    #[error("#ERROR: Malformed formula")]
    ParseError,
    #[error("#ERROR: Incompatible types, expected {0}")]
    TypeMismatch(&'static str),
    #[error("#ERROR: Incompatible types")]
    BinaryTypeMismatch,
    #[error("#ERROR: This cell references an empty field")]
    InvalidReference,
    #[error("#ERROR: This operation takes {0:?} args, but {1} were supplied")]
    // TODO: Select a concrete Range impl and use that
    InvalidArgCount(core::ops::Bound<usize>, usize),
    #[error("#ERROR: Could not find an operation named {0}")]
    NoOpFound(String),
    #[error("#ERROR: Referenced cell {0} has errors {1:?}")]
    RefError(Box<CellError>, Position),
    #[error("#ERROR: These errors have occured in this formula: {0:?}")]
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

impl Sheet {
    // TODO: in case of reference cycles this implementation will cycle till the stack blows up, fix this
    pub fn resolve_refs(&mut self) {
        for iy in 0..self.cells.len() {
            for jx in 0..self.cells[iy].len() {
                self.resolve_on_pos((jx, iy).into());
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
                let mut map = operators::get_form_map();

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

impl Into<SerdeValue> for Sheet {
    fn into(self) -> SerdeValue {
        let mut map: SerdeMap<String, SerdeValue> = SerdeMap::new();
        map.insert("id".to_string(), SerdeValue::String(self.id.clone()));
        
        let data = self
            .cells
            .iter()
            .map(|row| {
                SerdeValue::Array(row
                    .iter()
                    .map(|cell| cell.clone().into())
                    .collect::<Vec<SerdeValue>>()
                )
            })
            .collect::<Vec<_>>();
        let data = SerdeValue::Array(data);
        map.insert("data".to_owned(), data);
        
        SerdeValue::Object(map)
    }
}

impl jsonway::Serializer for Sheet {
    fn build(&self, json: &mut ObjectBuilder) {
        json.set("id", self.id.clone());
        json.array("data", |j_row| {
            self.cells.iter().for_each(|row| {
                j_row.array(|j_cell| {
                    row.iter().for_each(|cell| {
                        j_cell.push_json(cell.clone().into());
                    });
                })
            });
        });
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
                    operators::get_form_map()
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
