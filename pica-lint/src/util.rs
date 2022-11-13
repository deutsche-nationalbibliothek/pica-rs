use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use flate2::read::GzDecoder;

pub(crate) fn reader<P: AsRef<Path>>(
    path: P,
) -> io::Result<BufReader<Box<dyn Read>>> {
    let path = path.as_ref();

    let reader: Box<dyn Read> =
        match path.extension().and_then(OsStr::to_str) {
            Some("gz") => Box::new(GzDecoder::new(File::open(path)?)),
            _ => {
                if path.to_str() != Some("-") {
                    Box::new(File::open(path)?)
                } else {
                    Box::new(io::stdin())
                }
            }
        };

    Ok(BufReader::new(reader))
}
