use sway_types::Span;

use crate::semantic_analysis::TyScrutinee;

pub(crate) struct ReachableReport {
    pub(crate) reachable: bool,
    pub(crate) span: Span,
}

impl ReachableReport {
    pub(super) fn new(reachable: bool, scrutinee: TyScrutinee) -> ReachableReport {
        ReachableReport {
            reachable,
            span: scrutinee.span,
        }
    }
}
