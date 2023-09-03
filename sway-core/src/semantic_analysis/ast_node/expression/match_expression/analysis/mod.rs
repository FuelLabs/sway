mod constructor_factory;
mod matrix;
mod patstack;
mod pattern;
mod range;
mod reachable_report;
mod usefulness;
mod witness_report;
mod duplicates;

pub(in crate::semantic_analysis::ast_node::expression) use reachable_report::ReachableReport;
pub(crate) use usefulness::check_match_expression_usefulness;
pub(crate) use duplicates::collect_duplicate_match_pattern_variables;
