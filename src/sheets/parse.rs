//! Contains parsers for formulas

use nom::{
    branch::alt,
    bytes::{
        complete::take_while,
        complete::{tag, take_while_m_n},
    },
    character::complete::digit1,
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{pair, tuple},
};

use super::{num::Num, CellError, Expr, OpInfo, Position};

type VerboseResult<I, O, E> = Result<(I, O), nom::Err<VerboseError<E>>>;

// NOTE: this parser does not consider `543` to be a float
// anything which matches `[0-9]+\.[0-9]+` is considered to be a float
fn parse_float(i: &str) -> VerboseResult<&str, Num, &'_ str> {
    let num = map(digit1, |str: &str| str);

    let dot_and_after = tuple((tag("."), digit1));

    map(tuple((num, dot_and_after)), |(num, (_, after))| {
        Num::F((String::from(num) + "." + after).parse().unwrap())
    })(i)
}

fn parse_int(i: &str) -> VerboseResult<&str, Num, &'_ str> {
    map(digit1, |s: &str| Num::I(s.parse().unwrap()))(i)
}

fn parse_num(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    map(alt((parse_float, parse_int)), |num| num.into())(i)
}

fn parse_ref(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    let letter = map(
        take_while_m_n(1, 1, |ix: char| ix.is_alphabetic()),
        |c: &str| c.bytes().next().unwrap(),
    );

    let numbers1 = map(digit1, |s: &str| s.parse::<usize>().unwrap());

    map(tuple((letter, numbers1)), |(x, y)| {
        let pos = Position {
            x: (x as u8 - b'A') as usize,
            y: y,
        };
        if pos.y == 0 {
            CellError::InvalidReference.into()
        } else {
            Position {
                y: pos.y - 1,
                ..pos
            }
            .into()
        }
    })(i)
}

fn parse_str(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    map(
        tuple((tag("\""), take_while(|c| c != '"'), tag("\""))),
        |(_, s, _): (_, &str, _)| Expr::Value(Box::new(s.to_owned())),
    )(i)
}

fn parse_bool(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    map(alt((tag("false"), tag("true"))), |s: &str| {
        Expr::Value(Box::new(s.starts_with('t')))
    })(i)
}

/// TODO: this solution is recursive and thus has the ability to blow up the stack on some large data, maybe fix this?
// TODO: whitespace
fn parse_fn(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    let name = take_while(|c| c != '(');

    let parse_all = alt((parse_bool, parse_num, parse_ref, parse_str, parse_fn));

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

pub fn parse_entry(i: &str) -> VerboseResult<&str, Expr, &'_ str> {
    match i.starts_with('=') {
        false => map(
            alt((
                parse_bool,
                |s: &str| Ok(("", Expr::Value(Box::new(s.to_owned())))))),
            |expr| expr,
        )(i),
        true => map(
            pair(
                tag("="),
                alt::<_, _, _, _>((parse_bool, parse_num, parse_ref, parse_str, parse_fn)),
            ),
            |(_, expr)| expr,
        )(i),
    }
}

#[cfg(test)]
mod tests {
    use crate::sheets::{parse::*, *};

    impl Expr {
        fn as_value_unchecked(&self) -> &dyn Value {
            match self {
                Expr::Value(v) => v.as_ref(),
                _ => panic!("Not a Value"),
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
            Expr::Value(Box::new("lol".to_owned()))
                .as_value_unchecked()
                .downcast_ref()
        );
    }

    #[test]
    fn test_num() {
        assert_eq!(("", Num::I(531).into()), parse_num("531").unwrap());

        assert_eq!(("", Num::F(6.1).into()), parse_num("6.1").unwrap())
    }

    #[test]
    fn test_parse_form() {
        assert_eq!(
            parse_fn("SUM(A1,52)")
                .expect("test with fn does not fail")
                .1,
            Expr::Form(OpInfo {
                name: "SUM".to_owned(),
                args: vec![
                    Expr::Ref(Position { x: 0, y: 0 }),
                    Expr::Value(Box::new(Num::I(52)))
                ]
            })
        );

        assert_eq!(
            parse_entry("=SUM(A1,52)").unwrap().1,
            Expr::Form(OpInfo {
                name: "SUM".to_owned(),
                args: vec![
                    Expr::Ref(Position { x: 0, y: 0 }),
                    Expr::Value(Box::new(Num::I(52)))
                ]
            })
        );
    }

    #[test]
    fn test_parse_from_nested() {
        assert_eq!(
            parse_entry("=SUM(A1,MUL(5, B2))").unwrap().1,
            Expr::Form(OpInfo {
                name: "SUM".to_owned(),
                args: vec![
                    Expr::Ref(Position { x: 0, y: 0 }),
                    Expr::Form(OpInfo {
                        name: "MUL".to_owned(),
                        args: vec![
                            Expr::Value(Box::new(Num::I(5))),
                            Expr::Ref(Position { x: 1, y: 1 })
                        ]
                    })
                ]
            })
        );
    }

    #[test]
    fn parse_str_inside_form() {
        assert_eq!(
            parse_entry("=CONCAT(\"H\", \"i\")").unwrap().1,
            Expr::Form(OpInfo {
                name: "CONCAT".to_owned(),
                args: vec![
                    Expr::Value(Box::new("H".to_owned())),
                    Expr::Value(Box::new("i".to_owned())),
                ]
            })
        );
    }

    #[test]
    fn parse_err() {
        let raw: RawCellData = RawCellData::String("=SUM(".to_owned());
        assert_eq!(Expr::from(raw), Expr::Err(CellError::ParseError))
    }
}
