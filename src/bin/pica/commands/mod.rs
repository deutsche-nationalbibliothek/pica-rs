pub mod cat;
mod common;
pub mod filter;
pub mod json;
pub mod print;
pub mod sample;

use crate::util::App;
pub(crate) use common::Config;

pub fn subcmds() -> Vec<App> {
    vec![
        filter::cli(),
        print::cli(),
        cat::cli(),
        sample::cli(),
        json::cli(),
    ]
}
