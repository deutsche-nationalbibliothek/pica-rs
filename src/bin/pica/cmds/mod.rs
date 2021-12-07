pub(crate) mod cat;
pub(crate) mod completion;
pub(crate) mod filter;
pub(crate) mod frequency;
pub(crate) mod invalid;
pub(crate) mod json;
pub(crate) mod partition;
pub(crate) mod print;
pub(crate) mod sample;
pub(crate) mod select;
pub(crate) mod slice;
pub(crate) mod split;
pub(crate) mod xml;

use crate::util::App;

pub(crate) fn subcmds() -> Vec<App> {
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
        slice::cli(),
        split::cli(),
        xml::cli(),
    ]
}
