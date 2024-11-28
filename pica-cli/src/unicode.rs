use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(
    Debug,
    PartialEq,
    Default,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    clap::ValueEnum,
)]
pub(crate) enum NormalizationForm {
    #[default]
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
    fn nf_from_str() {
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
    fn nf_to_string() {
        assert_eq!(NormalizationForm::Nfc.to_string(), "nfc");
        assert_eq!(NormalizationForm::Nfkc.to_string(), "nfkc");
        assert_eq!(NormalizationForm::Nfd.to_string(), "nfd");
        assert_eq!(NormalizationForm::Nfkd.to_string(), "nfkd");
    }
}
