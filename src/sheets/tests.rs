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
            vec![
                RawCellData::Int(6),
                RawCellData::String("=C2".to_owned()),
            ],
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
                vec![
                    Num::I(6).into(),
                    CellError::InvalidReference.into(),
                ]
            ]
        }
    )
}

// #[test]
// fn parse_then_resolve_with_forms() {
//     let raw = RawSheet {
//         id: "sheet-test".to_owned(),
//         data: vec![vec![
//             RawCellData::Int(5),
//             RawCellData::String("=A1".to_owned()),
//             RawCellData::String("=SUM(A1, B1".to_owned()),
//             RawCellData::String("=D1".to_owned()),
//             RawCellData::String("=G1".to_owned()),
//             RawCellData::String("=A1".to_owned()),
//         ]],
//     };

//     let mut sheet: Sheet = raw.into();
//     println!("{:?}", sheet);
//     sheet.resolve_refs();

//     assert_eq!(
//         sheet,
//         Sheet {
//             id: "sheet-test".to_owned(),
//             cells: vec![vec![
//                 Num::I(5).into(),
//                 Num::I(5).into(),
//                 Num::I(22).into(),
//                 Num::I(22).into(),
//                 Num::I(22).into(),
//                 Num::I(5).into(),
//                 Num::I(5).into(),
//             ]]
//         }
//     )
// }
