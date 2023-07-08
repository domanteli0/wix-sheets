#![allow(incomplete_features)]

pub mod num;
pub mod tests;
pub mod value;

use std::{fmt::Debug};
use thiserror::Error;
// use serde::{Serialize, Serializer};

use derive_more::From;
use nom::{
    branch::alt,
    bytes::{
        complete::take_while,
        complete::{tag, take_while_m_n},
    },
    character::complete::digit1,
    combinator::{map},
    error::VerboseError,
    multi::many0,
    sequence::{pair, tuple},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    id: String,
    cells: Vec<Vec<Expr>>,
}

#[derive(Debug, Clone, From, PartialEq)]
pub enum Expr {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo),
    Err(CellError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: char,
    pub y: u32,
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CellError {
    #[error("$ERROR: TODO")]
    ParseError,
    #[error("#ERROR: Incompatible types")]
    TypeMismatch,
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

// Once RFC2515 (https://github.com/rust-lang/rust/issues/63063) lands
// this could be turned into:
// `impl Fn(&str) -> Result<Box<dyn Value>, E>`
type CellParser<E> = Box<dyn Fn(&str) -> VerboseResult<&str, Expr, E>>;
use crate::data::*;

use self::{num::Num, value::Value};
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
            RawCellData::String(s) => match parse_entry(&s[..]) {
                Ok((_, expr)) => expr,
                Err(_) => Expr::Err(CellError::ParseError),
            },
        }
    }
}

type VerboseResult<I, O, E> = Result<(I, O), nom::Err<VerboseError<E>>>;

// TODO: this implementation could be improved
// and could instead return a `&str`
//
// NOTE: this parser does not consider `543` to be a float
// anything which matches `[0-9]+\.[0-9]+` is considered to be a float
fn float1S(i: &str) -> VerboseResult<&str, Num, &'_ str> {
    let num = map(digit1, |str: &str| str);

    let dot_and_after = tuple((tag("."), digit1));

    map(tuple((num, dot_and_after)), |(num, (_, after))| {
        Num::F((String::from(num) + after).parse().unwrap())
    })(i)
}

fn digit1S(i: &str) -> VerboseResult<&str, Num, &'_ str> {
    map(digit1, |s: &str| Num::I(s.parse().unwrap()))(i)
}

fn parse_num(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    map(alt((float1S, digit1S)), |num| num.into())(i)
}

fn parse_ref(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    let letter = map(
        take_while_m_n(1, 1, |ix: char| ix.is_alphabetic()),
        |c: &str| c.bytes().next().unwrap(),
    );

    let numbers1 = map(digit1, |s: &str| s.parse::<u32>().unwrap());

    map(tuple((letter, numbers1)), |(x, y)| {
        Position { x: x as char, y }.into()
    })(i)
}

fn parse_str(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    map(
        tuple((tag("\""), take_while(|c| c != '"'), tag("\""))),
        |(_, s, _): (_, &str, _)| Expr::Value(Box::new(s.to_owned())),
    )(i)
}

/// TODO: this solution is recursive and thus has the ability to blow up the stack on some large data, maybe fix this?
// TODO: whitespace
fn parse_fn(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    let name = take_while(|c| c != '(');

    let parse_all = alt((parse_num, parse_ref, parse_str, parse_fn));

    let list_elem = map(
        pair(parse_all, take_while(|c| c == ' ' || c == ',')),
        |(expr, _)| expr,
    );

    let args = map(
        tuple((
            pair(tag("("), take_while(|c| c == ' ')),
            many0(list_elem),
            tag(")"),
        )),
        |(_, exprs, _)| exprs,
    );

    map(pair(name, args), |(name, args)| {
        Expr::Form(OpInfo {
            name: name.to_owned(),
            args,
        })
    })(i)
}

fn parse_entry(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    match i.starts_with('=') {
        false => Ok(("", Expr::Value(Box::new(i.to_owned())))),
        true => map(
            pair(
                tag("="),
                alt::<_, _, _, _>((parse_num, parse_ref, parse_str, parse_fn)),
            ),
            |(_, expr)| expr,
        )(i),
    }
}
