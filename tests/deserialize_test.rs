use wix_sheets::data::RawData;

#[test]
fn parses() {
    let str = include_str!("example.json");
    let data: Result<RawData, _> = serde_json::from_str(str);
    assert!(data.is_ok());
}