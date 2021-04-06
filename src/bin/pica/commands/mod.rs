use crate::cli::App;

pub(crate) mod cat;
pub(crate) mod completion;
pub(crate) mod frequency;
pub(crate) mod invalid;

pub fn commands() -> Vec<App> {
    vec![
        cat::cli(),
        completion::cli(),
        frequency::cli(),
        invalid::cli(),
    ]
}
