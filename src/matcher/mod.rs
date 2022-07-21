mod common;
mod field_matcher;
mod record_matcher;
mod subfield_list_matcher;
mod subfield_matcher;

pub use common::{BooleanOp, ComparisonOp};
pub use field_matcher::FieldMatcher;
pub use record_matcher::RecordMatcher;
pub use subfield_list_matcher::SubfieldListMatcher;
pub use subfield_matcher::SubfieldMatcher;

pub(crate) use common::{
    parse_comparison_op_bstring, parse_comparison_op_usize,
};
pub(crate) use field_matcher::{
    parse_field_matcher, parse_field_matcher_exists,
};
pub(crate) use subfield_list_matcher::{
    parse_subfield_list_matcher, parse_subfield_list_matcher_singleton,
};
pub(crate) use subfield_matcher::parse_subfield_matcher;
