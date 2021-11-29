mod common;
mod flags;
mod occurrence_matcher;
mod subfield_list_matcher;
mod subfield_matcher;
mod tag_matcher;

pub use common::{BooleanOp, ComparisonOp};
pub use flags::MatcherFlags;
pub use occurrence_matcher::OccurrenceMatcher;
pub use subfield_list_matcher::SubfieldListMatcher;
pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;

pub(crate) use common::{
    parse_comparison_op_bstring, parse_comparison_op_usize,
};
pub(crate) use occurrence_matcher::parse_occurrence_matcher;
pub(crate) use subfield_list_matcher::{
    parse_subfield_list_matcher, parse_subfield_list_matcher_singleton,
};
pub(crate) use subfield_matcher::parse_subfield_matcher;
pub(crate) use tag_matcher::parse_tag_matcher;
