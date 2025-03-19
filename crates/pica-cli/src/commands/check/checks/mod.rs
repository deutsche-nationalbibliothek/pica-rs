mod datetime;
mod filter;
mod isni;
mod unicode;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "check")]
pub(crate) enum Checks {
    #[serde(rename = "datetime")]
    DateTime(Box<datetime::DateTime>),
    Filter(Box<filter::Filter>),
    Isni(Box<isni::Isni>),
    Unicode(Box<unicode::Unicode>),
}
