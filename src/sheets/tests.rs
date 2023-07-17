//! This is a module for tests

use super::*;

#[test]
fn parse_then_resolve_with_refs() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
                vec![Num::I(6).into(), CellError::InvalidReference((2, 1).into()).into(),]
            ]
        }
    )
}

#[test]
fn parse_then_resolve_ops_with_consts() {
    let mut ops = operators::get_default_op_map();
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![vec![RawCellData::String("=SUM(1, 2)".to_owned())]],
    };

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
    let mut ops = operators::get_default_op_map();
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![RawCellData::String("=SUM(A2, B2)".to_owned())],
            vec![RawCellData::Int(6), RawCellData::String("=1".to_owned())],
        ],
    };

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
fn parse_then_resolve_forms_with_nested_forms_with_refs() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();

    let sheet = sheet.resolve_refs(&mut ops);

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
    let mut ops = operators::get_default_op_map();
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![vec![RawCellData::String("=SUM(1, A2, \"Hi\")".to_owned())]],
    };

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![vec![CellError::FormError(vec![
                CellError::ArgError(1, Box::new(CellError::InvalidReference((0, 1).into()))),
                CellError::ArgError(2, Box::new(CellError::TypeMismatch("Num"))),
            ])
            .into()]]
        }
    )
}

#[test]
fn parse_then_resolve_forms_with_mul() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
fn parse_then_resolve_divide() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Num::I(4).into(), Num::I(8).into(),],
                vec![CellError::FormError(vec![CellError::DivByZero]).into()]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_with_concat() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value("Hello".to_owned().into()),
                    Expr::Value(", ".to_owned().into()),
                    Expr::Value("Hi!".to_owned().into()),
                ],
                vec![
                    Expr::Value("World".to_owned().into()),
                    Expr::Value("Hello, World!".to_owned().into()),
                    Expr::Value("!".to_owned().into()),
                    Expr::Value("Hello, World!".to_owned().into()),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_not() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![Expr::Value(true.into()), Expr::Value(false.into()),],
                vec![
                    Expr::Value(true.into()),
                    Expr::Value(false.into()),
                    Expr::Value(true.into()),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_gt() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Value(Num::I(5).into()),
                    Expr::Value(Num::F(6.0).into())
                ],
                vec![Expr::Value(false.into()), Expr::Value(true.into()),]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_eq() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
                    Expr::Value(true.into()),
                    Expr::Err(CellError::FormError(vec![CellError::BinaryTypeMismatch])),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_and_not() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
                    Expr::Value(true.into()),
                    Expr::Err(CellError::FormError(vec![CellError::BinaryTypeMismatch])),
                ],
                vec![
                    Expr::Value(true.into()),
                    Expr::Value(true.into()),
                ]
            ]
        }
    );
}

#[test]
fn parse_then_resolve_if() {
    let mut ops = operators::get_default_op_map();
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

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

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
                    Expr::Value("Equal".to_owned().into()),
                    Expr::Value(false.into()),
                ],
            ]
        }
    );
}

#[test]
fn ref_cycles_simple() {
    let mut ops = operators::get_default_op_map();
    let raw = RawSheet {
        id: "sheet-test".to_owned(),
        data: vec![
            vec![
                RawCellData::String("=B1".to_owned()),
                RawCellData::String("=A1".to_owned()),
            ],
            vec![
                RawCellData::String("=B1".to_owned()),
                RawCellData::String("=A1".to_owned()),
            ],
        ],
    };

    let sheet: Sheet = raw.into();
    let sheet = sheet.resolve_refs(&mut ops);

    let cycle_error = CellError::RefCycle(vec![(0, 0).into(), (1, 0).into()]);

    assert_eq!(
        sheet,
        Sheet {
            id: "sheet-test".to_owned(),
            cells: vec![
                vec![
                    Expr::Err(cycle_error.clone()),
                    Expr::Err(cycle_error.clone()),
                ],
                vec![
                    Expr::Err(CellError::RefError(Box::new(cycle_error.clone()), (1, 0).into())),
                    Expr::Err(CellError::RefError(Box::new(cycle_error.clone()), (0, 0).into())),
                ],
            ]
        }
    );
}
