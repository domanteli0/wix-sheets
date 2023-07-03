pub mod data;
pub mod sheets;

use std::error::Error;

use crate::data::RawData;
use serde_json;
use reqwest;

const HUB_URL_GET: &'static str = 
    "https://www.wix.com/_serverless/hiring-task-spreadsheet-evaluator/sheets";

fn main() -> Result<(), Box<dyn Error>> {
    let data_str = reqwest::blocking::get(HUB_URL_GET)?
        .text()?;

    let data_raw: RawData = serde_json::from_str(&data_str)?;
    println!("{:?}", data_raw);



    Ok(())
}
