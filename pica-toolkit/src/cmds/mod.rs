pub(crate) mod cat;
pub(crate) mod completions;
pub(crate) mod count;
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

use crate::util::Command;

pub(crate) fn subcmds() -> Vec<Command> {
    vec![
        cat::cli(),
        completions::cli(),
        count::cli(),
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