pub mod data;

use crate::data::RawData;
use serde_json;

const HUB_URL: &'static str = 
    "https://www.wix.com/_serverless/hiring-task-spreadsheet-evaluator";

fn main() {
    let data_str = include_str!("../tests/example.json");
    let data_raw: RawData = serde_json::from_str(data_str).unwrap();
    
    println!("{:?}", data_raw);
}
