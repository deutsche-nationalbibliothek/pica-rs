use crate::Subfield;
use std::fmt;

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

impl fmt::Display for Field {
    /// Format a field.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field = Field::new(
    ///     "012A",
    ///     None,
    ///     vec![
    ///         Subfield::new('a', "123"),
    ///         Subfield::new('b', "456"),
    ///         Subfield::new('c', "789"),
    ///     ],
    /// );
    ///
    /// assert_eq!(field.to_string(), "012A $a 123 $b 456 $c 789");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        if let Some(ref occurrence) = self.occurrence {
            write!(f, "/{}", occurrence)?;
        }

        if self.subfields.len() > 0 {
            write!(
                f,
                " {}",
                self.subfields
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join(" ")
            )?;
        }

        Ok(())
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

    #[test]
    fn test_fmt() {
        let field = Field::new("012A", None, vec![]);
        assert_eq!(format!("{}", field), "012A");

        let field = Field::new("012A", Some("01"), vec![]);
        assert_eq!(format!("{}", field), "012A/01");

        let field =
            Field::new("012A", Some("00"), vec![Subfield::new('a', "123")]);
        assert_eq!(format!("{}", field), "012A/00 $a 123");

        let field = Field::new(
            "012A",
            None,
            vec![Subfield::new('a', "123"), Subfield::new('b', "456")],
        );
        assert_eq!(format!("{}", field), "012A $a 123 $b 456");
    }
}
