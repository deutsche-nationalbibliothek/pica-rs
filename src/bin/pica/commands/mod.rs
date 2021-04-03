use crate::cli::App;

pub(crate) mod cat;
pub(crate) mod invalid;

pub fn commands() -> Vec<App> {
    vec![invalid::cli(), cat::cli()]
}
