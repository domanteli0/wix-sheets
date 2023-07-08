#![feature(trait_upcasting)]
#![feature(downcast_unchecked)]
#![allow(incomplete_features)]

pub mod data;
pub mod Data; // rust-analyser acts funky without this line
pub mod sheets;

use std::{error::Error, any::Any};

use crate::{data::RawData, sheets::{Expr, num::Num}};
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
