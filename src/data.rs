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

impl RawSheet {
    pub fn resolve_consts(&mut self) {
        self.data
            .iter_mut()
            .for_each(|row| {
                row.iter_mut()
                    .for_each(RawCellData::resolve_const)
            })
    }
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

    // TODO: Add tests for this as this does not occur in `example.json`
    // but is possible according to the spec
    pub fn resolve_const(&mut self) {
        if let Some(s) = self.get_str() {
            if let Ok(i) = s.trim_start_matches('=').parse() {
                self.swap(RawCellData::Int(i));
            } else if let Ok(f) = s.trim_start_matches('=').parse() {
                self.swap(RawCellData::Float(f));
            } else if s == "=false" {
                self.swap(RawCellData::Bool(false))
            } else if s == "=true" {
                self.swap(RawCellData::Bool(true))
            }
        }
    }
}