mod datetime;
mod filter;
mod isni;
mod iso639;
mod jel;
mod link;
mod unicode;

pub(crate) const fn strsim_threshold() -> f64 {
    0.8
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "check")]
pub(crate) enum Checks {
    #[serde(rename = "datetime")]
    DateTime(Box<datetime::DateTime>),
    Filter(Box<filter::Filter>),
    Isni(Box<isni::Isni>),
    #[serde(rename = "iso639-2b")]
    Iso639(Box<iso639::Iso639>),
    Jel(jel::Jel),
    Link(Box<link::Link>),
    Unicode(Box<unicode::Unicode>),
}
