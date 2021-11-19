use std::ops::Deref;

use crate::util::CliResult;
use bstr::BString;
use csv::ReaderBuilder;

#[derive(Debug, Default)]
pub struct FilterList(Vec<BString>);

impl Deref for FilterList {
    type Target = Vec<BString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FilterList {
    pub(crate) fn new(filenames: Option<clap::Values<'_>>) -> CliResult<Self> {
        let filenames: Vec<&str> = filenames.unwrap_or_default().collect();
        let mut ids: Vec<BString> = Vec::new();

        for filename in filenames {
            let mut reader = ReaderBuilder::new()
                .has_headers(false)
                .from_path(filename)?;

            for result in reader.byte_records() {
                let row = result.expect("valid csv row");
                let id = BString::from(row.get(0).expect("idn in column 1"));

                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }

        Ok(Self(ids))
    }
}
