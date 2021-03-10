use pica::{Field, Record};
use std::ops::Deref;

use crate::concept::Concept;

pub struct TopicalTerm<'a>(pub(crate) Record<'a>);

impl<'a> Deref for TopicalTerm<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TopicalTerm<'a> {
    fn get_label(&self, field: &Field) -> Option<String> {
        let mut result = String::new();

        if let Some(term) = field.first('a') {
            result.push_str(&term);
            result = result.replace('@', "");

            let addition = field.all('g');
            if !addition.is_empty() {
                result.push_str(&format!(" ({})", addition.join(", ")));
            }

            let subdivision = field.all('x');
            if !subdivision.is_empty() {
                result.push_str(&format!(" / {}", subdivision.join(" / ")));
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> Concept for TopicalTerm<'a> {
    fn idn(&self) -> String {
        self.first("003@").unwrap().first('0').unwrap()
    }

    fn pref_label(&self) -> Option<String> {
        if let Some(field) = self.first("041A") {
            self.get_label(&field)
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<String> {
        let mut result = Vec::new();

        for field in self.all("041@") {
            if let Some(label) = self.get_label(&field) {
                result.push(label)
            }
        }

        result
    }
}
