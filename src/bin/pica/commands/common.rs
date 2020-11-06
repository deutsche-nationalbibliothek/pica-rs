use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
pub struct Config;

impl Config {
    pub fn new() -> Self {
        Self {}
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
