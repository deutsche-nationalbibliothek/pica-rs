use extendr_api::prelude::*;
use libR_sys::Rf_error;
use pica::{Outcome, ReaderBuilder, Selectors};
use std::io::Cursor;
use std::os::raw;

pub fn unwrap_result<T, E>(val: std::result::Result<T, E>) -> T
where
    E: std::fmt::Display,
{
    match val {
        Ok(obj) => obj,
        Err(err) => {
            let msg = format!("{}", err);
            unsafe {
                Rf_error(msg.as_ptr() as *const raw::c_char);
            }
            unreachable!("Code should be unreachable after call R error");
        }
    }
}

/// Return string `"Hello world!"` to R.
/// @export
#[extendr]
fn pica_select(
    filename: &str,
    selectors: &str,
    header: Option<String>,
) -> Robj {
    let mut reader = unwrap_result(
        ReaderBuilder::new().skip_invalid(true).from_path(filename),
    );

    let selectors = unwrap_result(Selectors::decode(selectors));

    let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));
    println!("header = {:?}", header);

    for result in reader.records() {
        let record = unwrap_result(result);
        let outcome = selectors
            .iter()
            .map(|selector| record.select(selector, false))
            .fold(Outcome::default(), |acc, x| acc * x);

        for row in outcome.iter() {
            if !row.iter().all(|col| col.is_empty()) {
                unwrap_result(
                    wtr.write_record(
                        &row.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                    ),
                );
            }
        }
    }

    let inner = unwrap_result(wtr.into_inner());
    let buffer = inner.into_inner();

    let result = unwrap_result(String::from_utf8(buffer));
    result.into()
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod pica4r;
    fn pica_select;
}
