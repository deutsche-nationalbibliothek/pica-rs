use std::fmt::{self, Display};
use std::str::FromStr;

use unicode_normalization::UnicodeNormalization;

use crate::matcher::{ParseMatcherError, RecordMatcher};

pub struct RecordMatcherBuilder {
    normalization: Option<NormalizationForm>,
    matcher: RecordMatcher,
}

impl RecordMatcherBuilder {
    pub fn new<S: AsRef<str>>(
        matcher: S,
        normalization: Option<NormalizationForm>,
    ) -> Result<Self, ParseMatcherError> {
        let matcher = RecordMatcher::new(&translit(
            matcher,
            normalization.as_ref(),
        ))?;
        Ok(Self {
            matcher,
            normalization,
        })
    }

    pub fn and<S: AsRef<str>>(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher &= RecordMatcher::new(&translit(
                predicate,
                self.normalization.as_ref(),
            ))?;
        }

        Ok(self)
    }

    pub fn or<S: AsRef<str>>(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher |= RecordMatcher::new(&translit(
                predicate,
                self.normalization.as_ref(),
            ))?;
        }

        Ok(self)
    }

    pub fn not<S: AsRef<str>>(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher &= !RecordMatcher::new(&translit(
                predicate,
                self.normalization.as_ref(),
            ))?;
        }

        Ok(self)
    }

    pub fn build(self) -> RecordMatcher {
        self.matcher
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum NormalizationForm {
    #[default]
    #[cfg_attr(feature = "serde", serde(alias = "nfc"))]
    Nfc,
    #[cfg_attr(feature = "serde", serde(alias = "nfkc"))]
    Nfkc,
    #[cfg_attr(feature = "serde", serde(alias = "nfd"))]
    Nfd,
    #[cfg_attr(feature = "serde", serde(alias = "nfkd"))]
    Nfkd,
}

#[inline(always)]
fn translit<S>(s: S, normalizion: Option<&NormalizationForm>) -> String
where
    S: AsRef<str>,
{
    match normalizion {
        Some(NormalizationForm::Nfc) => s.as_ref().nfc().collect(),
        Some(NormalizationForm::Nfkc) => s.as_ref().nfkc().collect(),
        Some(NormalizationForm::Nfd) => s.as_ref().nfd().collect(),
        Some(NormalizationForm::Nfkd) => s.as_ref().nfkd().collect(),
        None => s.as_ref().into(),
    }
}

impl FromStr for NormalizationForm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nfc" => Ok(Self::Nfc),
            "nfkc" => Ok(Self::Nfkc),
            "nfd" => Ok(Self::Nfd),
            "nfkd" => Ok(Self::Nfkd),
            _ => Err(()),
        }
    }
}

impl Display for NormalizationForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nfc => write!(f, "nfc"),
            Self::Nfkc => write!(f, "nfkc"),
            Self::Nfd => write!(f, "nfd"),
            Self::Nfkd => write!(f, "nfkd"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_nf_from_str() {
        assert_eq!(NormalizationForm::Nfc, "nfc".parse().unwrap());
        assert!("NFC".parse::<NormalizationForm>().is_err());
        assert_eq!(NormalizationForm::Nfkc, "nfkc".parse().unwrap());
        assert!("NFKC".parse::<NormalizationForm>().is_err());
        assert_eq!(NormalizationForm::Nfd, "nfd".parse().unwrap());
        assert!("NFD".parse::<NormalizationForm>().is_err());
        assert_eq!(NormalizationForm::Nfkd, "nfkd".parse().unwrap());
        assert!("NFKD".parse::<NormalizationForm>().is_err());
    }

    #[test]
    fn test_nf_to_string() {
        assert_eq!(NormalizationForm::Nfc.to_string(), "nfc");
        assert_eq!(NormalizationForm::Nfkc.to_string(), "nfkc");
        assert_eq!(NormalizationForm::Nfd.to_string(), "nfd");
        assert_eq!(NormalizationForm::Nfkd.to_string(), "nfkd");
    }

    #[test]
    fn test_translit() {
        use NormalizationForm::*;

        assert_eq!(
            translit("Am\u{0e9}lie", Some(&Nfc)),
            "Am\u{0e9}lie"
        );
        assert_eq!(
            translit("Am\u{0e9}lie", Some(&Nfkc)),
            "Am\u{0e9}lie"
        );
        assert_eq!(
            translit("Am\u{0e9}lie", Some(&Nfd)),
            "Ame\u{301}lie"
        );
        assert_eq!(
            translit("Am\u{0e9}lie", Some(&Nfkd)),
            "Ame\u{301}lie"
        );
        assert_eq!(
            translit("Ame\u{301}lie", Some(&Nfd)),
            "Ame\u{301}lie"
        );
        assert_eq!(
            translit("Ame\u{301}lie", Some(&Nfkd)),
            "Ame\u{301}lie"
        );
        assert_eq!(
            translit("Ame\u{301}lie", Some(&Nfc)),
            "Am\u{0e9}lie"
        );
        assert_eq!(
            translit("Ame\u{301}lie", Some(&Nfkc)),
            "Am\u{0e9}lie"
        );

        assert_eq!(translit("Ame\u{301}lie", None), "Ame\u{301}lie");
        assert_eq!(translit("Am\u{0e9}lie", None), "Am\u{0e9}lie");
    }
}
