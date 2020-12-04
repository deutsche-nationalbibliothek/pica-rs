pub mod cat;
mod common;
pub mod filter;
pub mod json;
pub mod print;
pub mod sample;
pub mod select;
pub mod split;

use crate::util::App;
pub(crate) use common::Config;

pub fn subcmds() -> Vec<App> {
    vec![
        cat::cli(),
        filter::cli(),
        json::cli(),
        print::cli(),
        sample::cli(),
        select::cli(),
        split::cli(),
    ]
}
