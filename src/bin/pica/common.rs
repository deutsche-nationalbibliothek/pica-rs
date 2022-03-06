use std::collections::BTreeSet;
use std::ops::Deref;

use crate::util::CliResult;
use bstr::BString;
use csv::ReaderBuilder;

#[derive(Debug, Default)]
pub struct FilterList(BTreeSet<BString>);

impl Deref for FilterList {
    type Target = BTreeSet<BString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FilterList {
    pub(crate) fn new(filenames: Vec<&str>) -> CliResult<Self> {
        let mut ids: BTreeSet<BString> = BTreeSet::new();

        for filename in filenames {
            let mut reader = ReaderBuilder::new()
                .has_headers(false)
                .from_path(filename)?;

            for result in reader.byte_records() {
                let row = result.expect("valid csv row");
                ids.insert(BString::from(row.get(0).expect("idn in column 1")));
            }
        }

        Ok(Self(ids))
    }
}
