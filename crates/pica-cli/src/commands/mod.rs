#[cfg(feature = "unstable")]
pub(crate) use check::Check;
pub(crate) use completions::Completions;
pub(crate) use concat::Concat;
pub(crate) use config::Config;
pub(crate) use convert::Convert;
pub(crate) use count::Count;
pub(crate) use explode::Explode;
pub(crate) use filter::Filter;
pub(crate) use frequency::Frequency;
pub(crate) use hash::Hash;
pub(crate) use invalid::Invalid;
pub(crate) use partition::Partition;
pub(crate) use print::Print;
pub(crate) use sample::Sample;
pub(crate) use select::Select;
pub(crate) use slice::Slice;
pub(crate) use split::Split;

#[cfg(feature = "unstable")]
mod check;
mod completions;
mod concat;
mod config;
mod convert;
mod count;
mod explode;
mod filter;
mod frequency;
mod hash;
mod invalid;
mod partition;
mod print;
mod sample;
mod select;
mod slice;
mod split;
