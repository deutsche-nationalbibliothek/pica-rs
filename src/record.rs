use crate::Field;

#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<Field>,
}
