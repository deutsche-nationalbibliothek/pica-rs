use pica::{Field, Record};
use std::ops::Deref;

use crate::concept::Concept;

pub struct Event<'a>(pub(crate) Record<'a>);

impl<'a> Deref for Event<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Event<'a> {
    fn get_label(&self, field: &Field) -> Option<String> {
        let mut result = String::new();
        let mut parens = String::new();
        let check = vec!['n', 'd', 'c', 'g'];

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            if !check.contains(&subfield.code()) {
                if !parens.is_empty() {
                    result.push_str(&format!(" ({})", parens));
                    parens.clear();
                }
            }

            match subfield.code() {
                'a' => {
                    result.push_str(&value.replace('@', ""));
                }
                'x' | 'b' => {
                    result.push_str(&format!(" / {}", value));
                }
                'g' => {
                    if parens.is_empty() {
                        result.push_str(&format!(" ({})", value))
                    } else {
                        parens.push_str(&format!(" ({})", value))
                    }
                }
                'n' | 'd' | 'c' => {
                    if !parens.is_empty() {
                        parens.push_str(", ");
                    }
                    parens.push_str(&value);
                }
                _ => continue,
            }
        }

        if !parens.is_empty() {
            result.push_str(&format!(" ({})", parens));
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> Concept for Event<'a> {
    fn idn(&self) -> String {
        self.first("003@").unwrap().first('0').unwrap()
    }

    fn pref_label(&self) -> Option<String> {
        if let Some(field) = self.first("030A") {
            self.get_label(&field)
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<String> {
        let mut result = Vec::new();

        for field in self.all("030@") {
            if let Some(label) = self.get_label(&field) {
                result.push(label)
            }
        }

        result
    }
}
