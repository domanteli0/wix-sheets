#![feature(trait_upcasting)]
#![feature(downcast_unchecked)]
#![allow(incomplete_features)]

pub mod data;
pub mod Data; // rust-analyser acts funky without this line
pub mod sheets;

use std::{convert::{ From, Into }, error::Error, any::Any};

use serde_json;
use reqwest;
use wix_sheets::{data::{RawCellData, RawSheet, RawData}, sheets::{num::Num, Expr, Sheet}};
const HUB_URL_GET: &'static str = 
    "https://www.wix.com/_serverless/hiring-task-spreadsheet-evaluator/sheets";

fn main() -> Result<(), Box<dyn Error>> {
    let data_str = reqwest::blocking::get(HUB_URL_GET)?
        .text()?;

    let data_raw: RawData = serde_json::from_str(&data_str)?;
    println!("{:?}", data_raw);

    let mut data: Vec<Sheet> = data_raw.sheets.into_iter().map(Into::<_>::into).collect();
    println!("{:?}", data);

    for s in &mut data {
        s.resolve_refs();
    }

    println!("{:#?}", data);

    Ok(())
}
