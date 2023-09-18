use std::str::FromStr;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum NormalizationForm {
    #[serde(alias = "nfc")]
    Nfc,
    #[serde(alias = "nfkc")]
    Nfkc,
    #[serde(alias = "nfd")]
    Nfd,
    #[serde(alias = "nfkd")]
    Nfkd,
}

impl FromStr for NormalizationForm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nfc" => Ok(Self::Nfc),
            "nfkc" => Ok(Self::Nfkc),
            "nfd" => Ok(Self::Nfd),
            "nfkd" => Ok(Self::Nfkd),
            _ => Err(format!("invalid normalization form '{s}'")),
        }
    }
}

impl NormalizationForm {
    pub fn translit<S>(self, value: S) -> String
    where
        S: AsRef<str>,
    {
        match self {
            Self::Nfc => value.as_ref().nfc().collect::<String>(),
            Self::Nfkc => value.as_ref().nfkc().collect::<String>(),
            Self::Nfd => value.as_ref().nfd().collect::<String>(),
            Self::Nfkd => value.as_ref().nfkd().collect::<String>(),
        }
    }

    pub fn translit_opt<S>(value: S, nf: Option<Self>) -> String
    where
        S: AsRef<str>,
    {
        match nf {
            Some(nf) => nf.translit(value),
            None => value.as_ref().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translit() {
        use NormalizationForm::*;

        assert_eq!(Nfc.translit("Am\u{0e9}lie"), "Am\u{0e9}lie");
        assert_eq!(Nfkc.translit("Am\u{0e9}lie"), "Am\u{0e9}lie");
        assert_eq!(Nfd.translit("Am\u{0e9}lie"), "Ame\u{301}lie");
        assert_eq!(Nfkd.translit("Am\u{0e9}lie"), "Ame\u{301}lie");
        assert_eq!(Nfd.translit("Ame\u{301}lie"), "Ame\u{301}lie");
        assert_eq!(Nfkd.translit("Ame\u{301}lie"), "Ame\u{301}lie");
        assert_eq!(Nfc.translit("Ame\u{301}lie"), "Am\u{0e9}lie");
        assert_eq!(Nfkc.translit("Ame\u{301}lie"), "Am\u{0e9}lie");
    }

    #[test]
    fn test_from_str() {
        use NormalizationForm::*;

        assert_eq!(NormalizationForm::from_str("nfc").unwrap(), Nfc);
        assert_eq!(NormalizationForm::from_str("nfkc").unwrap(), Nfkc);
        assert_eq!(NormalizationForm::from_str("nfd").unwrap(), Nfd);
        assert_eq!(NormalizationForm::from_str("nfkd").unwrap(), Nfkd);

        assert!(NormalizationForm::from_str("foo").is_err());
        assert!(NormalizationForm::from_str("").is_err());
    }
}
