use unicode::Unicode;

mod unicode;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "check")]
pub(crate) enum Checks {
    Unicode(Unicode),
}
