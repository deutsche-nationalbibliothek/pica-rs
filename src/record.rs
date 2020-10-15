use crate::Field;

#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a> {
    pub fields: Vec<Field<'a>>,
}
