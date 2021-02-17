extern crate pica;

use pica::legacy::Record;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let filename = env::args()
        .skip(1)
        .next()
        .unwrap_or("tests/data/1.dat".to_string());

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match Record::decode(&line.unwrap()) {
            Ok(record) => println!("{}", record.pretty()),
            Err(_) => eprintln!("invalid record!"),
        }
    }
}
