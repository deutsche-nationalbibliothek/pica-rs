extern crate pica;

use bstr::io::BufReadExt;
use pica::Record;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let filename = env::args()
        .skip(1)
        .next()
        .unwrap_or("tests/data/1.dat".to_string());

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for result in reader.byte_lines() {
        let line = result.unwrap();

        match Record::from_bytes(&line) {
            Ok(record) => println!("{}", record.pretty()),
            Err(_) => eprintln!("invalid record!"),
        }
    }
}
