use bstr::ByteSlice;
use pica_record::prelude::*;
use unicode_normalization::*;

use crate::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Unicode {
    normalization: Option<NormalizationForm>,
}

impl Unicode {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        if record.validate().is_err() {
            return (true, None);
        }

        if let Some(ref nf) = self.normalization {
            let r#fn = match nf {
                NormalizationForm::Nfc => is_nfc,
                NormalizationForm::Nfkc => is_nfkc,
                NormalizationForm::Nfd => is_nfd,
                NormalizationForm::Nfkd => is_nfkd,
            };

            for field in record.fields() {
                for subfield in field.subfields() {
                    let value = subfield.value().to_str().unwrap();
                    if !r#fn(value) {
                        let comment =
                            format!("invalid value '{value}'");
                        return (true, Some(comment));
                    }
                }
            }
        }

        (false, None)
    }
}
