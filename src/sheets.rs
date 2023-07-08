#![allow(incomplete_features)]

pub mod num;
pub mod value;

use std::fmt::Debug;
// use serde::{Serialize, Serializer};

use derive_more::From;
use nom::{
    bytes::{
        complete::take_while,
        complete::{take_while_m_n, tag},
    },
    character::complete::{digit1},
    combinator::{map, opt},
    sequence::{tuple, pair}, branch::alt, multi::many0, error::VerboseError,
};

pub struct Sheet<'a> {
    id: String,
    cells: Vec<Vec<Expr<'a>>>,
}

#[derive(Debug, Clone, From, PartialEq)]
pub enum Expr<'a> {
    Value(Box<dyn Value>),
    Ref(Position),
    Form(OpInfo<'a>),
    Err(CellError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: char,
    pub y: u32,
}

// TODO: A better error type
type CellError = String;

#[derive(Debug, Clone, PartialEq)]
pub struct OpInfo<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

type CellParser<E> = Box<dyn Fn(&str) -> Result<Box<dyn Value>, E>>;
use crate::data::*;

use self::{num::Num, value::Value};
impl<'a> Sheet<'a> {
    // TODO: clean up this mess
    pub fn from_sheets_with_custom_parser<E>(
        mut value: RawSheet,
        parsers: CellParser<E>,
    ) -> Result<Self, E> {
        value.resolve_consts();
        let cells = value
            .data
            .into_iter()
            .map(|raw_row| {
                raw_row
                    .into_iter()
                    .map(move |raw_cell| -> Expr<'_> {
                        match raw_cell {
                            RawCellData::Int(i) =>
                                Expr::Value(Box::new(Num::I(i))),
                            RawCellData::Float(f) => {
                                // TODO: fix this
                                Expr::Value(Box::new(Num::I(f as i64)))
                            }
                            RawCellData::Bool(b) => Expr::Value(Box::new(b)),
                            RawCellData::String(s) => {
                                // TODO, parse formula cells here probably
                                todo!()
                            }
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(Self {
            id: value.id,
            cells,
        })
    }
}

type VerboseResult<I, O, E> = Result<(I, O), nom::Err<VerboseError<E>>>;

// TODO: this implementation could be improved
// and could instead returna `&str`
fn float1S(i: &str) -> VerboseResult<&str, String, &'_ str> {
    let num = 
    map(
        digit1,
        |str: &str| str
    );

    let dot_and_after = tuple((tag("."), digit1));

    map(
        tuple((num, opt(dot_and_after))),
        |(num, after)| {
            String::from(num) + after.map(|(_, s)| s).unwrap_or("")
        }
    )(i)
}


fn digit1S(i: &str) -> VerboseResult<&str, String, &'_ str> {
    map(
        digit1,
        ToOwned::to_owned
    )(i)
}

fn parse_const(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    map(
        tuple((tag("="), parse_num)),
        |(_, num) :(_, Expr<'_>)| { num }
    )(i)
}

fn parse_num(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    map(
        alt((digit1S, float1S)),
        |num :String| num.parse::<Num>().unwrap().into()
    )(i)
}

fn parse_ref(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    let letter = map(
        take_while_m_n(1, 1, 
            |ix: char| ix.is_alphabetic()
        ), 
        |c: &str| { c.bytes().next().unwrap() }
    );

    let numbers1 = map(
        digit1,
        |s: &str| s.parse::<u32>().unwrap()
    );

    map(
        tuple(( letter, numbers1 )),
        |(x, y)| {
            Position {x: x as char, y}.into()
        }
    )(i)
}

fn parse_str(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    map(
        tuple((
            tag("\""),
            take_while(|c| c != '"'),
            tag("\"")
        )),
        |(_, s, _) : (_, &str, _)| Expr::Value(Box::new(s.to_owned()))
    )(i)
}

/// TODO: this solution is recursive and thus has the ability to blow up the stack on some large data, maybe fix this?
// TODO: whitespace
fn parse_fn(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    let name =
        take_while(|c| c != '(');

    let parse_all = alt((
        parse_num,
        parse_ref,
        parse_str,
        parse_fn,
    ));


    let list_elem = map(
        pair(parse_all, take_while(|c| c == ' ' || c == ',')),
        |(expr, _)| expr
    );

    let args = map(
        tuple((
            pair(tag("("), take_while(|c| c == ' ' )),
            many0(list_elem),
            tag(")")
        )),
        |(_, exprs, _)| exprs
    );

    map(
        pair(name, args),
        |(name, args)| Expr::Form(OpInfo { name, args })
    )(i)
}

fn parse_entry(i: &str) -> VerboseResult<&str, Expr<'_>, &'_ str> {
    map(
        pair(
            tag("="), 
            alt::<_, _, _, _>((
                parse_const,
                parse_ref,
                parse_str,
                parse_fn,
            ))
        ),
        |(_, expr)| expr
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Expr<'_> {
        fn as_value_unchecked(&self) -> &dyn Value {
            match self {
                Expr::Value(v) => v.as_ref(),
                _ => panic!("Not a Value")
            }
        }
    }

    #[test]
    fn test_parse_str() {
        let str = "\"lol\"";
        let parsed = parse_str(str).unwrap();

        assert_eq!(parsed.0, "");

        assert_eq!(
            parsed.1.as_value_unchecked().downcast_ref::<String>(),
            Expr::Value(Box::new("lol".to_owned())).as_value_unchecked().downcast_ref()
        );

    }

    #[test]
    fn test_num() {
        let str = "531";
        let parsed = parse_num(str).unwrap();
        assert_eq!(parsed.0, "");
        assert_eq!(parsed.1, Num::I(531).into());
    }

    #[test]
    fn test_parse_fn() {
        assert_eq!(
            parse_fn("SUM(A1,52)").expect("test with fn does not fail").1,
            Expr::Form(
                OpInfo {
                    name: "SUM",
                    args: vec![
                        Expr::Ref( Position { x: 'A', y: 1 }),
                        Expr::Value(Box::new(Num::I( 52 )))
                    ]
                }
            )
        ); 
    }

    #[test]
    fn test_parse_form() {
        
        // parse_test("a").unwrap();
        // tag::<_, _, VerboseError<&str>>("=")("a").unwrap();

        assert_eq!(
            parse_entry("=SUM(A1,52)").unwrap().1,
            Expr::Form(
                OpInfo {
                    name: "SUM",
                    args: vec![
                        Expr::Ref( Position { x: 'A', y: 1 }),
                        Expr::Value(Box::new(Num::I( 52 )))
                    ]
                }
            )
        );
    }

    #[test]
    fn test_parse_from_nested() {
        assert_eq!(
            parse_entry("=SUM(A1,MUL(5, B2))").unwrap().1,
            Expr::Form(
                OpInfo {
                    name: "SUM",
                    args: vec![
                        Expr::Ref( Position { x: 'A', y: 1 }),
                        Expr::Form(
                            OpInfo {
                                name: "MUL",
                                args: vec![
                                    Expr::Value(Box::new( Num::I(5) )),
                                    Expr::Ref( Position { x: 'B', y: 2 })
                                ]
                            }
                        )
                    ]
                }
            )
        );
    }
}