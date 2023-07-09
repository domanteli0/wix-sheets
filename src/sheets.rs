#![allow(incomplete_features)]

pub mod num;
pub mod parse;
pub mod tests;
pub mod value;

use std::{fmt::Debug, collections::HashMap};
use thiserror::Error;
// use serde::{Serialize, Serializer};
use derive_more::{From, IsVariant, Unwrap};

use crate::data::{RawSheet, RawCellData};
use self::{num::Num, value::Value};

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
    #[error("#ERROR: Incompatible types")]
    TypeMismatch,
    #[error("#ERROR: This cell references an empty field")]
    InvalidReference,
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

pub type Form = Box<dyn Fn(OpInfo) -> Expr>;

impl Sheet {

    // fn get_form_map() -> HashMap<&'static str, Form> {

    //     let map = HashMap::from([
    //         ("SUM", |info| {
    //             let get_args = |op_info,| {

    //             }
    //         })
    //     ]);

    //     map
    // }

    // TODO: in case of reference cycles this implementation will cycle till the stack blows up, fix this
    pub fn resolve_refs(&mut self) {

        let mut iy = 0;
        while iy < self.cells.len() {
            let mut jx = 0;
            while jx < self.cells[iy].len() {
                self.resolve_on_pos(dbg!((jx, iy).into()));
                jx += 1;
            }
            iy += 1;
        }
    }

    fn resolve_on_pos(&mut self, pos: Position) -> Option<&Expr> {
        let expr = self
            .get(pos)?;

        let new_expr: Expr = match expr {
            Expr::Ref(r) => self
                .resolve_on_pos(*r)
                .map(|e| e.clone())
                .unwrap_or(CellError::InvalidReference.into()),
            Expr::Form(_) => todo!(),
            Expr::Value(v) => v.clone().into(),
            Expr::Err(e) => e.clone().into(),
        };
        
        self.set_unchecked(pos, new_expr);
        self.get(pos)
    }
}

impl<'a> From<RawSheet> for Sheet {
    fn from(mut value: RawSheet) -> Self {
        let cells = value
            .data
            .into_iter()
            .map(|raw_row|
                raw_row.into_iter().map(RawCellData::into).collect()
            )
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
