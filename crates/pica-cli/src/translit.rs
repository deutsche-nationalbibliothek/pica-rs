use std::fmt::{self, Display};
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(
    Debug, PartialEq, Default, Clone, Serialize, Deserialize, ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub(crate) enum NormalizationForm {
    #[default]
    Nfc,
    Nfkc,
    Nfd,
    Nfkd,
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

impl FromStr for NormalizationForm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nfc" => Ok(Self::Nfc),
            "nfkc" => Ok(Self::Nfkc),
            "nfd" => Ok(Self::Nfd),
            "nfkd" => Ok(Self::Nfkd),
            _ => Err("invalid normalizion form".into()),
        }
    }
}

pub(crate) fn translit<S: AsRef<str>>(
    nf: Option<NormalizationForm>,
) -> impl Fn(S) -> String {
    use NormalizationForm::*;

    match nf {
        Some(Nfc) => |s: S| s.as_ref().chars().nfc().to_string(),
        Some(Nfkc) => |s: S| s.as_ref().chars().nfkc().to_string(),
        Some(Nfd) => |s: S| s.as_ref().chars().nfd().to_string(),
        Some(Nfkd) => |s: S| s.as_ref().chars().nfkd().to_string(),
        None => |s: S| s.as_ref().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translit_nfc() {
        let r#fn = translit(Some(NormalizationForm::Nfc));
        assert_eq!(r#fn("Am\u{0e9}lie"), "Am\u{0e9}lie");
        assert_eq!(r#fn("Ame\u{301}lie"), "Am\u{0e9}lie");
    }

    #[test]
    fn test_translit_nfkc() {
        let r#fn = translit(Some(NormalizationForm::Nfkc));
        assert_eq!(r#fn("Am\u{0e9}lie"), "Am\u{0e9}lie");
        assert_eq!(r#fn("Ame\u{301}lie"), "Am\u{0e9}lie");
    }

    #[test]
    fn test_translit_nfd() {
        let r#fn = translit(Some(NormalizationForm::Nfd));
        assert_eq!(r#fn("Am\u{0e9}lie"), "Ame\u{301}lie");
        assert_eq!(r#fn("Ame\u{301}lie"), "Ame\u{301}lie");
    }

    #[test]
    fn test_translit_nfkd() {
        let r#fn = translit(Some(NormalizationForm::Nfd));
        assert_eq!(r#fn("Am\u{0e9}lie"), "Ame\u{301}lie");
        assert_eq!(r#fn("Ame\u{301}lie"), "Ame\u{301}lie");
    }

    #[test]
    fn test_translit_none() {
        let r#fn = translit(None);
        assert_eq!(r#fn("Am\u{0e9}lie"), "Am\u{0e9}lie");
        assert_eq!(r#fn("Ame\u{301}lie"), "Ame\u{301}lie");
    }

    #[test]
    fn test_nf_to_string() {
        assert_eq!(NormalizationForm::Nfc.to_string(), "nfc");
        assert_eq!(NormalizationForm::Nfkc.to_string(), "nfkc");
        assert_eq!(NormalizationForm::Nfd.to_string(), "nfd");
        assert_eq!(NormalizationForm::Nfkd.to_string(), "nfkd");
    }

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
}
