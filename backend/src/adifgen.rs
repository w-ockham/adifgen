use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct ADIFRecord {
    qso_date: String,
    time_on: String,
    band: String,
    mode: String,
    station: String,
    oprator: String,
    my_sig: Vec<(String, String)>,
    his_sig: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct ADIFData {
    status: String,
    records: Vec<ADIFRecord>,
}

fn parse_string(lines: &str) -> ADIFData {
    let mut status = "OK";
    if lines.contains("<ADIF_VER") {
        status = "NG";
    } else {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(lines.as_bytes());
        for result in rdr.records() {
            let record = result.unwrap();
            println!("{:?}", record)
        }
    }
    ADIFData {
        status: status.to_string(),
        records: [].to_vec(),
    }
}

pub fn adifcheck(
    activator_call: &str,
    operator: &str,
    my_qth: &str,
    references: &str,
    his_qth: &str,
    log: &str,
) -> ADIFData {
    parse_string(log)
}
