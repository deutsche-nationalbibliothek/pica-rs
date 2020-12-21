pub mod cat;
mod common;
pub mod completion;
pub mod filter;
pub mod frequency;
pub mod invalid;
pub mod json;
pub mod partition;
pub mod print;
pub mod sample;
pub mod select;
pub mod split;

use crate::util::App;
pub(crate) use common::Config;

pub fn subcmds() -> Vec<App> {
    vec![
        cat::cli(),
        completion::cli(),
        filter::cli(),
        frequency::cli(),
        invalid::cli(),
        json::cli(),
        partition::cli(),
        print::cli(),
        sample::cli(),
        select::cli(),
        split::cli(),
    ]
}
