mod datetime;
mod filter;
mod unicode;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "check")]
pub(crate) enum Checks {
    #[serde(rename = "datetime")]
    DateTime(Box<datetime::DateTime>),
    Filter(Box<filter::Filter>),
    Unicode(Box<unicode::Unicode>),
}
