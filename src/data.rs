use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct RawData {
    #[serde(rename = "submissionUrl")]
    pub submission_url: String,
    pub sheets: Vec<RawSheet>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RawSheet {
    pub id: String,
    pub data: Vec<Vec<RawCellData>>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RawCellData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool)
}

impl RawCellData {
    fn get_str(&self) -> Option<&str> {
        if let RawCellData::String(s) = self { Some(s) }
        else { None }
    }

    fn swap(&mut self, mut other: Self) {
        std::mem::swap(self, &mut other);
    }
}
