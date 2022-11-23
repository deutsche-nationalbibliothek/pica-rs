use bstr::BString;
use pica_record::ByteRecord;
use serde::Deserialize;

use self::checksum::Checksum;
use self::date::Date;
use self::filter::Filter;
use self::iri::Iri;
use self::iso639::Iso639;
use self::orcid::Orcid;
use self::refcheck::RefCheck;
use self::unicode::Unicode;

mod checksum;
mod date;
mod filter;
mod iri;
mod iso639;
mod orcid;
mod refcheck;
mod unicode;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Lints {
    Checksum(Checksum),
    Date(Date),
    Filter(Filter),
    RefCheck(RefCheck),
    Iri(Iri),
    Iso639(Iso639),
    Orcid(Orcid),
    Unicode(Unicode),
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Status {
    Postponed,
    Hit,
    #[default]
    Miss,
}

impl From<bool> for Status {
    fn from(value: bool) -> Self {
        if value {
            Status::Hit
        } else {
            Status::Miss
        }
    }
}

pub trait Lint {
    fn preprocess(&mut self, _record: &ByteRecord) {}
    fn check(&mut self, record: &ByteRecord) -> Status;
    fn finish(&mut self) -> Vec<(BString, Status)> {
        vec![]
    }
}

impl Lint for Lints {
    fn check(&mut self, record: &ByteRecord) -> Status {
        match self {
            Self::Checksum(ref mut l) => l.check(record),
            Self::Date(ref mut l) => l.check(record),
            Self::Filter(ref mut l) => l.check(record),
            Self::Iri(ref mut l) => l.check(record),
            Self::Iso639(ref mut l) => l.check(record),
            Self::Orcid(ref mut l) => l.check(record),
            Self::RefCheck(ref mut l) => l.check(record),
            Self::Unicode(ref mut l) => l.check(record),
        }
    }

    fn preprocess(&mut self, record: &ByteRecord) {
        if let Self::RefCheck(ref mut l) = self {
            l.preprocess(record)
        };

        // match self {
        //     Self::RefCheck(ref mut l) => l.preprocess(record),
        //     _ => (),
        // }
    }

    fn finish(&mut self) -> Vec<(BString, Status)> {
        match self {
            Self::RefCheck(ref mut l) => l.finish(),
            _ => vec![],
        }
    }
}
