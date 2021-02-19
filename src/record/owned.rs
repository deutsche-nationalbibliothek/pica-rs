use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, PartialEq, Serialize)]
pub struct Subfield {
    #[serde(rename(serialize = "name"))]
    pub(crate) code: char,
    pub(crate) value: String,
}

impl Subfield {
    /// Returns the subfield code.
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the subfield value.
    pub fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Occurrence(pub(crate) String);

impl Deref for Occurrence {
    type Target = String;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Field {
    #[serde(rename(serialize = "name"))]
    pub(crate) tag: String,
    pub(crate) occurrence: Option<Occurrence>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Deref for Field {
    type Target = Vec<Subfield>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.subfields
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Record {
    pub(crate) fields: Vec<Field>,
}

impl Deref for Record {
    type Target = Vec<Field>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
