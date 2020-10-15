use crate::Field;

#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a, 'b, 'c> {
    pub fields: Vec<Field<'a, 'b, 'c>>,
}
