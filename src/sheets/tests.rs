use super::*;

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
    let str = "531";
    let parsed = parse_num(str).unwrap();
    assert_eq!(parsed.0, "");
    assert_eq!(parsed.1, Num::I(531).into());
}

#[test]
fn test_parse_fn() {
    assert_eq!(
        parse_fn("SUM(A1,52)")
            .expect("test with fn does not fail")
            .1,
        Expr::Form(OpInfo {
            name: "SUM".to_owned(),
            args: vec![
                Expr::Ref(Position { x: 'A', y: 1 }),
                Expr::Value(Box::new(Num::I(52)))
            ]
        })
    );
}

#[test]
fn test_parse_form() {
    assert_eq!(
        parse_entry("=SUM(A1,52)").unwrap().1,
        Expr::Form(OpInfo {
            name: "SUM".to_owned(),
            args: vec![
                Expr::Ref(Position { x: 'A', y: 1 }),
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
                Expr::Ref(Position { x: 'A', y: 1 }),
                Expr::Form(OpInfo {
                    name: "MUL".to_owned(),
                    args: vec![
                        Expr::Value(Box::new(Num::I(5))),
                        Expr::Ref(Position { x: 'B', y: 2 })
                    ]
                })
            ]
        })
    );
}

#[test]
fn parse_err() {
    let raw: RawCellData = RawCellData::String("=SUM(".to_owned());
    assert_eq!(Expr::from(raw), Expr::Err(CellError::ParseError))
}
