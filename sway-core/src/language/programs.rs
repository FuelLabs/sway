use std::sync::Arc;

use super::{lexed::LexedProgram, parsed::ParseProgram, ty::TyProgram};
use crate::semantic_analysis::program::TypeCheckFailed;
use sway_utils::PerformanceData;

/// Contains the lexed, parsed, typed compilation stages of a program, as well
/// as compilation metrics.
#[derive(Clone, Debug)]
pub struct Programs {
    pub lexed: Arc<LexedProgram>,
    pub parsed: Arc<ParseProgram>,
    pub typed: Result<Arc<TyProgram>, TypeCheckFailed>,
    pub metrics: PerformanceData,
}

impl Programs {
    pub fn new(
        lexed: Arc<LexedProgram>,
        parsed: Arc<ParseProgram>,
        typed: Result<Arc<TyProgram>, TypeCheckFailed>,
        metrics: PerformanceData,
    ) -> Programs {
        Programs {
            lexed,
            parsed,
            typed,
            metrics,
        }
    }
}
