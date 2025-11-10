mod allow;
mod datetime;
mod duplicates;
mod filter;
mod isni;
mod iso639;
mod jelc;
mod link;
mod unicode;

pub(crate) const fn strsim_threshold() -> f64 {
    0.8
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "check")]
pub(crate) enum Checks {
    Allow(Box<allow::Allow>),
    #[serde(rename = "datetime")]
    DateTime(Box<datetime::DateTime>),
    Duplicates(Box<duplicates::Duplicates>),
    Filter(Box<filter::Filter>),
    Isni(Box<isni::Isni>),
    #[serde(rename = "iso639-2b")]
    Iso639(Box<iso639::Iso639>),
    #[serde(rename = "jelc")]
    Jel(Box<jelc::Jel>),
    Link(Box<link::Link>),
    Unicode(Box<unicode::Unicode>),
}
