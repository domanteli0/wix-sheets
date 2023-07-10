use super::*;

#[test]
fn parse_then_resolve_with_refs() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::Int(5),
                RawCellData::String("=A1".to_owned()),
                RawCellData::Int(22),
                RawCellData::String("=C1".to_owned()),
                RawCellData::String("=D1".to_owned()),
                RawCellData::String("=G1".to_owned()),
                RawCellData::String("=A2".to_owned()),
            ],
            vec![RawCellData::Int(6), RawCellData::String("=C2".to_owned())],
        ],
    };

    let mut sheet: Sheet = raw.into();
    println!("{:?}", sheet);
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Num::I(5).into(),
                    Num::I(5).into(),
                    Num::I(22).into(),
                    Num::I(22).into(),
                    Num::I(22).into(),
                    Num::I(6).into(),
                    Num::I(6).into(),
                ],
                vec![Num::I(6).into(), CellError::InvalidReference.into(),]
            ]
        }
    )
}

#[test]
fn parse_then_resolve_ops_with_consts() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![vec![RawCellData::String("=SUM(1, 2)".to_owned())]],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![vec![Num::I(3).into()]]
        }
    );
}

#[test]
fn parse_then_resolve_forms_with_refs() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::String("=SUM(A2, B2)".to_owned())],
            vec![RawCellData::Int(6), RawCellData::String("=1".to_owned())],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(7).into()],
                vec![Num::I(6).into(), Num::I(1).into(),]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_forms_with_nested_froms_with_refs() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=SUM(A2, B2)".to_owned()),
                RawCellData::String("=SUM(A1, SUM(A3, B3))".to_owned()),
            ],
            vec![RawCellData::Int(6), RawCellData::String("=1".to_owned())],
            vec![
                RawCellData::String("=6.1".to_owned()).into(),
                RawCellData::String("=5".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(7).into(), Num::F(18.1).into(),],
                vec![Num::I(6).into(), Num::I(1).into(),],
                vec![Num::F(6.1).into(), Num::I(5).into(),]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_fn_with_errs() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![vec![RawCellData::String("=SUM(1, A2, \"Hi\")".to_owned())]],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![vec![CellError::FormError(vec![
                (2, CellError::InvalidReference),
                (3, CellError::TypeMismatch("Num")),
            ])
            .into()]]
        }
    )
}

#[test]
fn parse_then_resolve_forms_with_mul() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=MULTIPLY(2, 2)".to_owned()),
                RawCellData::String("=MULTIPLY(2, A1)".to_owned()),
            ],
            vec![RawCellData::String(
                "=SUM(MULTIPLY(A1, B1), 2.5)".to_owned(),
            )],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(4).into(), Num::I(8).into(),],
                vec![Num::F(34.5).into()]
            ]
        }
    )
}

#[test]
fn parse_then_resolve_divive() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=MULTIPLY(2, 2)".to_owned()),
                RawCellData::String("=MULTIPLY(2, A1)".to_owned()),
            ],
            vec![RawCellData::String(
                "=DIVIDE(MULTIPLY(A1, B1), 5)".to_owned(),
            )],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(4).into(), Num::I(8).into(),],
                vec![Num::F(32.0 / 5.0).into()]
            ]
        }
    );

    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=MULTIPLY(2, 2)".to_owned()),
                RawCellData::String("=MULTIPLY(2, A1)".to_owned()),
            ],
            vec![RawCellData::String(
                "=DIVIDE(MULTIPLY(A1, B1), 0)".to_owned(),
            )],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(4).into(), Num::I(8).into(),],
                vec![CellError::DivByZero.into(),]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_with_concat() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("Hello".to_owned()),
                RawCellData::String(", ".to_owned()),
                RawCellData::String("=\"Hi!\"".to_owned()),
            ],
            vec![
                RawCellData::String("=\"World\"".to_owned()),
                RawCellData::String("=CONCAT(A1, B1, A2, C2)".to_owned()),
                RawCellData::String("=CONCAT(\"!\")".to_owned()),
                RawCellData::String("=CONCAT(\"Hello, \", \"World!\")".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Box::new(String::from("Hello"))),
                    Expr::Value(Box::new(String::from(", "))),
                    Expr::Value(Box::new(String::from("Hi!"))),
                ],
                vec![
                    Expr::Value(Box::new(String::from("World"))),
                    Expr::Value(Box::new(String::from("Hello, World!"))),
                    Expr::Value(Box::new(String::from("!"))),
                    Expr::Value(Box::new(String::from("Hello, World!"))),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_not() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=NOT(false)".to_owned()),
                RawCellData::String("=NOT(A1)".to_owned()),
            ],
            vec![
                RawCellData::String("=NOT(B2))".to_owned()),
                RawCellData::String("false".to_owned()),
                RawCellData::String("=true".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Expr::Value(Box::new(true)), Expr::Value(Box::new(false)),],
                vec![
                    Expr::Value(Box::new(true)),
                    Expr::Value(Box::new(false)),
                    Expr::Value(Box::new(true)),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_gt() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::Int(5), RawCellData::Float(6.0)],
            vec![
                RawCellData::String("=GT(A1, B1)".to_owned()),
                RawCellData::String("=GT(B1, 4.9)".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Num::I(5).into()),
                    Expr::Value(Num::F(6.0).into())
                ],
                vec![Expr::Value(Box::new(false)), Expr::Value(Box::new(true)),]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_eq() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::Int(6), RawCellData::Float(6.0)],
            vec![
                RawCellData::String("=EQ(A1, B1)".to_owned()),
                RawCellData::String("=EQ(B1, \"String\")".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Num::I(6).into()),
                    Expr::Value(Num::F(6.0).into())
                ],
                vec![
                    Expr::Value(Box::new(true)),
                    Expr::Err(CellError::BinaryTypeMismatch),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_and_not() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::Int(6), RawCellData::Float(6.0)],
            vec![
                RawCellData::String("=EQ(A1, B1)".to_owned()),
                RawCellData::String("=EQ(B1, \"String\")".to_owned()),
            ],
            vec![
                RawCellData::String("=AND(true, A2)".to_owned()),
                RawCellData::String("=OR(false, A3)".to_owned()),
            ]
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Num::I(6).into()),
                    Expr::Value(Num::F(6.0).into())
                ],
                vec![
                    Expr::Value(Box::new(true)),
                    Expr::Err(CellError::BinaryTypeMismatch),
                ],
                vec![
                    Expr::Value(Box::new(true)),
                    Expr::Value(Box::new(true)),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_if() {
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::Int(6), RawCellData::Float(6.0)],
            vec![
                RawCellData::String("=IF(EQ(A1, B1), \"Equal\", \"Not equal\")".to_owned()),
                RawCellData::String("=EQ(A2, \"String\")".to_owned()),
            ],
        ],
    };

    let mut sheet: Sheet = raw.into();
    sheet.resolve_refs();

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Num::I(6).into()),
                    Expr::Value(Num::F(6.0).into())
                ],
                vec![
                    Expr::Value(Box::new("Equal".to_owned())),
                    Expr::Value(Box::new(false)),
                ],
            ]
        }
    );
}

