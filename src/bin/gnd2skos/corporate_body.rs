use pica::{Field, Record};
use std::ops::Deref;

use crate::concept::Concept;

pub struct CorporateBody<'a>(pub(crate) Record<'a>);

impl<'a> Deref for CorporateBody<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> CorporateBody<'a> {
    fn get_label(&self, field: &Field) -> Option<String> {
        let mut result = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();
            match subfield.code() {
                'a' => {
                    result.push_str(&value.replace('@', ""));
                }
                'g' => {
                    result.push_str(&format!(" ({})", value));
                }
                'x' | 'b' => {
                    result.push_str(&format!(" / {}", value));
                }
                'n' => {
                    result.push_str(&format!(", {}", value));
                }
                _ => continue,
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> Concept for CorporateBody<'a> {
    fn idn(&self) -> String {
        self.first("003@").unwrap().first('0').unwrap()
    }

    fn pref_label(&self) -> Option<String> {
        if let Some(field) = self.first("029A") {
            self.get_label(&field)
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<String> {
        let mut result = Vec::new();

        for field in self.all("029@") {
            if let Some(label) = self.get_label(&field) {
                result.push(label)
            }
        }

        result
    }
}
