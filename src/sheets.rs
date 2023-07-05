use std::{any::Any, fmt::Display};
use serde::{Serialize, Serializer};

pub struct Sheet<'a> {
    id: &'a str,
    cells: Vec<Vec<Expr<'a>>>,
}

pub enum Expr<'a> {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo<'a>),
    Err(CellError),
}

pub trait Value: Any + Display {
    fn clone_to_box(&self) -> Box<dyn Value>;
}

pub struct Position {
    pub x: char,
    pub y: u8,
}

type CellError = String;

pub struct OpInfo<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

// This newtype allows to change the underlying implementation
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Num(f64);


use crate::data::*;
impl<'a> From<RawData> for Sheet<'a> {
    fn from(mut value: RawData) -> Self {
        value.sheets.iter_mut().for_each(RawSheet::resolve_consts);

        // Self {
        //     id: &value.submission_url,
        //     cells
        // }

        todo!()
    }
}
