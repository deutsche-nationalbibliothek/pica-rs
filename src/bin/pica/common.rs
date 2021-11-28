use std::ops::Deref;

use crate::util::CliResult;
use bstr::BString;
use csv::ReaderBuilder;
use pica::matcher::TagMatcher;
use pica::{
    FieldMatcher, OccurrenceMatcher, RecordMatcher, SubfieldListMatcher,
    SubfieldMatcher, Tag,
};

#[derive(Debug, Default)]
pub struct FilterList(Vec<BString>, bool);

impl Deref for FilterList {
    type Target = Vec<BString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FilterList {
    pub(crate) fn new(filenames: Vec<&str>, invert: bool) -> CliResult<Self> {
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

        Ok(Self(ids, invert))
    }
}

impl From<FilterList> for RecordMatcher {
    fn from(list: FilterList) -> Self {
        Self::Singleton(Box::new(FieldMatcher::Subield(
            TagMatcher::Some(Tag::new("003@").unwrap()),
            OccurrenceMatcher::None,
            SubfieldListMatcher::Singleton(SubfieldMatcher::In(
                vec!['0'],
                list.0,
                list.1,
            )),
        )))
    }
}
