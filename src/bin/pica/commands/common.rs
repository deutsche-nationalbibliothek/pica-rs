use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

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
            Some(filename) => {
                let path = Path::new(filename);

                let reader: Box<dyn Read> =
                    if path.extension() == Some(OsStr::new("gz")) {
                        Box::new(GzDecoder::new(File::open(path)?))
                    } else {
                        Box::new(File::open(path)?)
                    };

                Box::new(BufReader::new(reader))
            }
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
