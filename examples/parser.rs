extern crate pica;

use std::env;
use std::fs::File;
use std::io::BufReader;

use bstr::io::BufReadExt;
use pica::StringRecord;

fn main() {
    let filename = env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/data/1.dat".to_string());

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for result in reader.byte_lines() {
        let line = result.unwrap();

        match StringRecord::from_bytes(line) {
            Ok(record) => println!("{record}"),
            Err(_) => eprintln!("invalid record!"),
        }
    }
}
