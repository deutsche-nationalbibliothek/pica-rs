mod cat;
mod completions;
mod count;
mod filter;
mod frequency;
mod invalid;
mod json;
mod partition;

pub(crate) use cat::{Cat, CatConfig};
pub(crate) use completions::Completions;
pub(crate) use count::{Count, CountConfig};
pub(crate) use filter::{Filter, FilterConfig};
pub(crate) use frequency::{Frequency, FrequencyConfig};
pub(crate) use invalid::Invalid;
pub(crate) use json::{Json, JsonConfig};
pub(crate) use partition::{Partition, PartitionConfig};

// pub(crate) mod print;
// pub(crate) mod sample;
// pub(crate) mod select;
// pub(crate) mod slice;
// pub(crate) mod split;
// pub(crate) mod xml;

// use crate::util::Command;

// pub(crate) fn subcmds() -> Vec<Command> {
//     vec![
//         cat::cli(),
//         completions::cli(),
//         count::cli(),
//         filter::cli(),
//         frequency::cli(),
//         invalid::cli(),
//         json::cli(),
//         partition::cli(),
//         print::cli(),
//         sample::cli(),
//         select::cli(),
//         slice::cli(),
//         split::cli(),
//         xml::cli(),
//     ]
// }
