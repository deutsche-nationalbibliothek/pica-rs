use crate::Subfield;

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub tag: String,
    pub occurrence: Option<String>,
    pub subfields: Vec<Subfield>,
}

impl Field {
    pub fn new<S>(
        tag: S,
        occurrence: Option<S>,
        subfields: Vec<Subfield>,
    ) -> Self
    where
        S: Into<String>,
    {
        let occurrence = match occurrence {
            Some(o) => Some(o.into()),
            None => None,
        };

        Self {
            tag: tag.into(),
            occurrence,
            subfields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let field = Field::new("003@", Some("00"), vec![]);
        assert_eq!(field.tag, "003@");
        assert_eq!(field.occurrence, Some("00".to_string()));
        assert!(field.subfields.is_empty());

        let field =
            Field::new("003@".to_string(), Some("".to_string()), vec![]);
        assert_eq!(field.tag, "003@");
        assert_eq!(field.occurrence, Some("".to_string()));
        assert!(field.subfields.is_empty());
    }
}
