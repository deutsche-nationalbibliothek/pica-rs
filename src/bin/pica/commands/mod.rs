use crate::cli::App;

pub(crate) mod invalid;

pub fn commands() -> Vec<App> {
    vec![invalid::cli()]
}
