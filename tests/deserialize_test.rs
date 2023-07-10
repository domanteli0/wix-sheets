use wix_sheets::data::{RawCellData::*, RawData};

#[test]
fn parses() {
    let str = include_str!("example.json");
    let data: RawData = serde_json::from_str(str).unwrap();

    assert_eq!(
        data
            .sheets
            .iter()
            .find(|s| s.id == "sheet-12")
            .unwrap()
            .data,
        vec![
            vec![Float(10.75), Float(10.75), String("=EQ(A1, B2)".into())],
            vec![Float(10.74), Float(10.74), String("=EQ(A2, B2)".into())]
        ]
    );
}
