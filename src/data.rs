use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawData {
    submition_url: Option<String>, // TODO: this should not be optional
    sheets: Vec<RawSheet>,
}

#[derive(Debug, Deserialize)]
pub struct RawSheet {
    id: String,
    data: Vec<Vec<CellData>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CellData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool)
}