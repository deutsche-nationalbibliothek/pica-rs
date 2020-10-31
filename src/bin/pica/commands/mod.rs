pub mod filter;
pub mod print;

use crate::util::App;

pub fn subcmds() -> Vec<App> {
    vec![filter::cli(), print::cli()]
}
