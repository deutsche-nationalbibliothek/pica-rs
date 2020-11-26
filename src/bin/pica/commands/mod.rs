// pub mod cat;
mod common;
pub mod filter;
// pub mod json;
pub mod print;
// pub mod sample;
// pub mod select;

use crate::util::App;
pub(crate) use common::Config;

pub fn subcmds() -> Vec<App> {
    vec![
        print::cli(),
        // cat::cli(),
        filter::cli(),
        /* json::cli(),
         * sample::cli(),
         * select::cli(), */
    ]
}
