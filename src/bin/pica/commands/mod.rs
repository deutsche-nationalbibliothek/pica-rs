pub mod cat;
mod common;
pub mod convert;
pub mod filter;
pub mod print;
pub mod sample;

use crate::util::App;
pub(crate) use common::Config;

pub fn subcmds() -> Vec<App> {
    vec![
        cat::cli(),
        convert::cli(),
        filter::cli(),
        print::cli(),
        sample::cli(),
    ]
}
