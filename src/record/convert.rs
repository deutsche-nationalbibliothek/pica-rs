use std::convert::From;

use crate::record::{borrowed, owned};

impl<'a> From<&borrowed::Subfield<'a>> for owned::Subfield {
    fn from(subfield: &borrowed::Subfield) -> Self {
        owned::Subfield {
            code: subfield.code,
            value: String::from_utf8(subfield.value.to_vec()).unwrap(),
        }
    }
}

impl<'a> From<&borrowed::Field<'a>> for owned::Field {
    fn from(field: &borrowed::Field) -> Self {
        owned::Field {
            tag: String::from_utf8(field.tag.to_vec()).unwrap(),
            occurrence: field
                .occurrence
                .map(|o| String::from_utf8(o.to_vec()).unwrap())
                .map(|o| owned::Occurrence(o)),
            subfields: field.iter().map(|s| owned::Subfield::from(s)).collect(),
        }
    }
}

impl<'a> From<borrowed::Record<'a>> for owned::Record {
    fn from(record: borrowed::Record) -> Self {
        Self {
            fields: record.iter().map(|f| owned::Field::from(f)).collect(),
        }
    }
}
