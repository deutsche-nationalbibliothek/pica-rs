use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug)]
pub struct Config;

impl Config {
    pub fn new() -> Self {
        Self {}
    }

    pub fn reader(
        &self,
        filename: Option<&str>,
    ) -> io::Result<Box<dyn BufRead>> {
        Ok(match filename {
            None => Box::new(BufReader::new(io::stdin())),
            Some(filename) => Box::new(BufReader::new(File::open(filename)?)),
        })
    }

    pub fn writer(
        &self,
        output: Option<&str>,
    ) -> io::Result<Box<dyn Write + 'static>> {
        Ok(match output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        })
    }
}
