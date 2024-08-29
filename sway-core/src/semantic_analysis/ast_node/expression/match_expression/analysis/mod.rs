mod constructor_factory;
mod match_pattern_variables;
mod matrix;
mod patstack;
mod pattern;
mod range;
mod reachable_report;
mod usefulness;
mod witness_report;

pub(crate) use match_pattern_variables::{
    collect_duplicate_match_pattern_variables, collect_match_pattern_variables,
};
pub(in crate::semantic_analysis::ast_node::expression) use reachable_report::ReachableReport;
pub(crate) use usefulness::check_match_expression_usefulness;
