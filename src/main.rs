#![feature(trait_upcasting)]
#![feature(downcast_unchecked)]
#![allow(incomplete_features)]

pub mod Data; // rust-analyser acts funky without this line
pub mod data;
pub mod sheets;

use std::{convert::Into, env, error::Error};
use std::collections::HashMap;

use jsonway::{ObjectBuilder, Serializer};
use reqwest;
use serde_json::{self, ser::Formatter};
use wix_sheets::{data::RawData, sheets::Sheet};

const HUB_URL_GET: &'static str =
    "https://www.wix.com/_serverless/hiring-task-spreadsheet-evaluator/sheets";

struct Results {
    email: String,
    results: Vec<Sheet>,
}
impl jsonway::Serializer for Results {
    fn build(&self, json: &mut ObjectBuilder) {
        json.set("email", self.email.clone());
        json.array("results", |j_array| {
            self.results.iter().for_each(|s| j_array.push_json(s.clone().into()))
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data_str = reqwest::blocking::get(HUB_URL_GET)?.text()?;

    let data_raw: RawData = serde_json::from_str(&data_str)?;
    let url = data_raw.submission_url.clone();
    println!("{:?}", data_raw);

    let mut data: Vec<Sheet> = data_raw.sheets.into_iter().map(Into::<_>::into).collect();
    println!("{:?}", data);

    for s in &mut data {
        s.resolve_refs();
    }

    println!("{:#?}", data);


    let mut results = Results {
        email: env::args().collect::<Vec<_>>().get(1).expect("no provided email").clone(),
        results: data,
    };

    println!("'{}'", results.serialize(true).to_string());


    let client = reqwest::blocking::Client::new();
    let req = client.post(url)
        .header("content-type", "application/json")
        .body(results.serialize(true).to_string())
        ;
    println!("{:#?}", &req);
    let res = req.send()?;

    println!("{:#?}", &res);
    println!("{}", res.text()?);

    // {"id":"sheet-14","error":"invalid result: invalid cell D1: expected value of type boolean, got string"}
    // {"id":"sheet-17","error":"invalid result: invalid cell D1: expected value of type boolean, got string"

    Ok(())
}
