use sway_types::Span;

use crate::semantic_analysis::TypedScrutinee;

pub(crate) struct ReachableReport {
    pub(crate) reachable: bool,
    pub(crate) span: Span,
}

impl ReachableReport {
    pub(super) fn new(reachable: bool, scrutinee: TypedScrutinee) -> ReachableReport {
        ReachableReport {
            reachable,
            span: scrutinee.span,
        }
    }
}
