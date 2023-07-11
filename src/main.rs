#![feature(trait_upcasting)]
#![allow(incomplete_features)]

pub mod Data; // rust-analyzer acts funky without this line
pub mod data;
pub mod sheets;

use std::{convert::Into, env, error::Error};

use jsonway::{ObjectBuilder, Serializer};
use reqwest;
use serde_json;
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
            self.results
                .iter()
                .for_each(|s| j_array.push_json(s.clone().into()))
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data_str = reqwest::blocking::get(HUB_URL_GET)?.text()?;

    // deserialize data
    let data_raw: RawData = serde_json::from_str(&data_str)?;
    let url = data_raw.submission_url.clone();

    // parse & compute fields
    let mut data: Vec<Sheet> = data_raw.sheets.into_iter().map(Into::<_>::into).collect();

    let mut ops = wix_sheets::sheets::operators::get_default_op_map();

    for s in &mut data {
        s.resolve_refs(&mut ops);
    }

    // serialize and send
    let mut results = Results {
        email: env::args()
            .collect::<Vec<_>>()
            .get(1)
            .expect("no provided email")
            .clone(),
        results: data,
    };

    let client = reqwest::blocking::Client::new();
    let req = client
        .post(url)
        .header("content-type", "application/json")
        .body(results.serialize(true).to_string());

    let res = req.send()?;

    // response
    println!("{:#?}", &res);
    println!("{}", res.text()?);

    Ok(())
}
