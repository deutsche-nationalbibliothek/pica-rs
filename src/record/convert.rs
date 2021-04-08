use std::convert::From;

use crate::record::{borrowed, owned};

impl<'a> From<&borrowed::Subfield> for owned::Subfield {
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
                .map(owned::Occurrence),
            subfields: field.iter().map(owned::Subfield::from).collect(),
        }
    }
}

impl<'a> From<borrowed::Record<'a>> for owned::Record {
    fn from(record: borrowed::Record) -> Self {
        Self {
            fields: record.iter().map(owned::Field::from).collect(),
        }
    }
}
