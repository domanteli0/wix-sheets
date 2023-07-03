use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct RawData {
    #[serde(rename = "submissionUrl")]
    submission_url: String,
    sheets: Vec<RawSheet>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RawSheet {
    id: String,
    data: Vec<Vec<RawCellData>>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RawCellData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool)
}