pub mod cat;
pub mod filter;
pub mod print;
pub mod sample;

use crate::util::App;

pub fn subcmds() -> Vec<App> {
    vec![filter::cli(), print::cli(), cat::cli(), sample::cli()]
}
